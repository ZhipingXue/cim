using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// gRPC service for module lifecycle state.
/// All control apps expose this service.
/// </summary>
public class ModuleStateGrpcService : GrpcServiceBase
{
    private readonly ModuleLifecycle.ModuleStateBase _module;

    public ModuleStateGrpcService(ModuleInfo moduleInfo, ModuleLifecycle.ModuleStateBase module, ILogger<ModuleStateGrpcService> logger)
        : base(moduleInfo, logger)
    {
        _module = module ?? throw new ArgumentNullException(nameof(module));
    }

    /// <summary>
    /// Gets the current module state.
    /// </summary>
    public Task<GetModuleStateResponse> GetStateAsync(GetModuleStateRequest request, ServerCallContext context)
    {
        if (!ValidateModuleId(request.ModuleId))
        {
            return Task.FromResult(new GetModuleStateResponse 
            { 
                Status = Error(1, "Invalid module ID"), 
                State = ModuleState.Error 
            });
        }

        return Task.FromResult(new GetModuleStateResponse
        {
            Status = Ok(),
            State = ToGrpcModuleState(_module.CurrentState),
            Timestamp = Google.Protobuf.WellKnownTypes.Timestamp.FromDateTime(DateTime.UtcNow)
        });
    }

    /// <summary>
    /// Streams module state changes to the client.
    /// </summary>
    public async Task StreamStateAsync(Empty request, IServerStreamWriter<GetModuleStateResponse> responseStream, ServerCallContext context)
    {
        Logger.LogInformation("Starting state stream for {ModuleId}", ModuleInfo.QualifiedId);
        var cts = CancellationTokenSource.CreateLinkedTokenSource(context.CancellationToken);

        void OnStateChanged(object? sender, ModuleLifecycle.ModuleStateChangedEventArgs args)
        {
            if (cts.Token.IsCancellationRequested) return;
            responseStream.WriteAsync(new GetModuleStateResponse 
            { 
                Status = Ok(), 
                State = ToGrpcModuleState(args.ToState),
                Timestamp = Google.Protobuf.WellKnownTypes.Timestamp.FromDateTime(DateTime.UtcNow)
            }).ConfigureAwait(false);
        }

        _module.StateChanged += OnStateChanged;
        try
        {
            // Send initial state
            await responseStream.WriteAsync(new GetModuleStateResponse 
            { 
                Status = Ok(), 
                State = ToGrpcModuleState(_module.CurrentState),
                Timestamp = Google.Protobuf.WellKnownTypes.Timestamp.FromDateTime(DateTime.UtcNow)
            });

            // Keep stream open until cancelled
            await Task.Delay(Timeout.Infinite, cts.Token);
        }
        catch (OperationCanceledException)
        {
            Logger.LogInformation("State stream cancelled for {ModuleId}", ModuleInfo.QualifiedId);
        }
        finally
        {
            _module.StateChanged -= OnStateChanged;
            cts.Dispose();
        }
    }
}
