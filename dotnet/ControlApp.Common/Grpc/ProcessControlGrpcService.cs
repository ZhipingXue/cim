using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// gRPC service for process state control.
/// Only process chambers expose this service.
/// </summary>
public class ProcessControlGrpcService : GrpcServiceBase
{
    private readonly StateMachine.IProcessStateControl _processControl;

    public ProcessControlGrpcService(
        ModuleInfo moduleInfo, 
        StateMachine.IProcessStateControl processControl,
        ILogger<ProcessControlGrpcService> logger)
        : base(moduleInfo, logger)
    {
        _processControl = processControl ?? throw new ArgumentNullException(nameof(processControl));
    }

    /// <summary>
    /// Sets the process state (transition).
    /// </summary>
    public async Task<SetProcessStateResponse> SetStateAsync(SetProcessStateRequest request, ServerCallContext context)
    {
        if (!ValidateModuleId(request.ModuleId))
        {
            return new SetProcessStateResponse 
            { 
                Status = Error(1, "Invalid module ID"), 
                State = request.State 
            };
        }

        var targetState = FromGrpcState(request.State);
        var result = await _processControl.TryTransitionToAsync(targetState, context.CancellationToken);

        return new SetProcessStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Transition failed"),
            State = ToGrpcState(_processControl.CurrentState)
        };
    }

    /// <summary>
    /// Starts processing with a recipe.
    /// </summary>
    public async Task<SetProcessStateResponse> StartProcessingAsync(string recipeName, ServerCallContext context)
    {
        var result = await _processControl.StartProcessingAsync(recipeName, context.CancellationToken);

        return new SetProcessStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Start processing failed"),
            State = ToGrpcState(_processControl.CurrentState)
        };
    }

    /// <summary>
    /// Completes the current processing.
    /// </summary>
    public async Task<SetProcessStateResponse> CompleteProcessingAsync(ServerCallContext context)
    {
        var result = await _processControl.CompleteProcessingAsync(context.CancellationToken);

        return new SetProcessStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Complete processing failed"),
            State = ToGrpcState(_processControl.CurrentState)
        };
    }

    /// <summary>
    /// Aborts the current processing.
    /// </summary>
    public async Task<SetProcessStateResponse> AbortProcessingAsync(ServerCallContext context)
    {
        var result = await _processControl.AbortProcessingAsync(context.CancellationToken);

        return new SetProcessStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Abort processing failed"),
            State = ToGrpcState(_processControl.CurrentState)
        };
    }

    private static StateMachine.ProcessState FromGrpcState(ProcessState state)
    {
        return state switch
        {
            ProcessState.Idle => StateMachine.ProcessState.Idle,
            ProcessState.Ready => StateMachine.ProcessState.Ready,
            ProcessState.Processing => StateMachine.ProcessState.Processing,
            ProcessState.Complete => StateMachine.ProcessState.Complete,
            ProcessState.Error => StateMachine.ProcessState.Error,
            _ => StateMachine.ProcessState.Idle
        };
    }

    private static ProcessState ToGrpcState(StateMachine.ProcessState state)
    {
        return state switch
        {
            StateMachine.ProcessState.Idle => ProcessState.Idle,
            StateMachine.ProcessState.Ready => ProcessState.Ready,
            StateMachine.ProcessState.Processing => ProcessState.Processing,
            StateMachine.ProcessState.Complete => ProcessState.Complete,
            StateMachine.ProcessState.Error => ProcessState.Error,
            _ => ProcessState.Unspecified
        };
    }
}
