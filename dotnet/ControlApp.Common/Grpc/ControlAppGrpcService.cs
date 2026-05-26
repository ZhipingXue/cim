using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// Unified gRPC service that implements the generated ControlAppServiceBase
/// and delegates to the individual service implementations.
/// </summary>
public class ControlAppGrpcService : ControlAppService.ControlAppServiceBase
{
    private readonly ModuleStateGrpcService _moduleStateService;
    private readonly ProcessControlGrpcService? _processControlService;
    private readonly RobotTransferGrpcService? _robotTransferService;
    private readonly SubstrateTransferGrpcService? _substrateTransferService;
    private readonly AlarmGrpcService _alarmService;
    private readonly ILogger<ControlAppGrpcService> _logger;

    public ControlAppGrpcService(
        ModuleStateGrpcService moduleStateService,
        AlarmGrpcService alarmService,
        ILogger<ControlAppGrpcService> logger,
        ProcessControlGrpcService? processControlService = null,
        RobotTransferGrpcService? robotTransferService = null,
        SubstrateTransferGrpcService? substrateTransferService = null)
    {
        _moduleStateService = moduleStateService ?? throw new ArgumentNullException(nameof(moduleStateService));
        _alarmService = alarmService ?? throw new ArgumentNullException(nameof(alarmService));
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        _processControlService = processControlService;
        _robotTransferService = robotTransferService;
        _substrateTransferService = substrateTransferService;
    }

    public override Task<GetModuleStateResponse> GetModuleState(GetModuleStateRequest request, ServerCallContext context)
        => _moduleStateService.GetStateAsync(request, context);

    public override Task StreamStateChanges(Empty request, IServerStreamWriter<GetModuleStateResponse> responseStream, ServerCallContext context)
        => _moduleStateService.StreamStateAsync(request, responseStream, context);

    public override async Task<SetProcessStateResponse> SetProcessState(SetProcessStateRequest request, ServerCallContext context)
    {
        if (_processControlService == null)
        {
            throw new RpcException(new Status(global::Grpc.Core.StatusCode.Unimplemented, "Process control not supported by this module"));
        }
        return await _processControlService.SetStateAsync(request, context);
    }

    public override async Task<SetRobotTransferStateResponse> SetRobotTransferState(SetRobotTransferStateRequest request, ServerCallContext context)
    {
        if (_robotTransferService == null)
        {
            throw new RpcException(new Status(global::Grpc.Core.StatusCode.Unimplemented, "Robot transfer not supported by this module"));
        }
        return await _robotTransferService.SetStateAsync(request.State, context);
    }

    public override Task<ReportAlarmResponse> ReportAlarm(ReportAlarmRequest request, ServerCallContext context)
        => _alarmService.ReportAlarmAsync(request, context);

    public override Task<PublishDataResponse> PublishData(PublishDataRequest request, ServerCallContext context)
    {
        // TODO: Implement data publishing via Zenohd
        _logger.LogInformation("PublishData: {Key} = {Value}", request.Key, request.Value);
        return Task.FromResult(new PublishDataResponse { Status = new global::Cim.Common.StatusCode { Code = 0, Message = "OK" } });
    }
}
