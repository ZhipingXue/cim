using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// gRPC service for substrate transfer state control.
/// Chambers and substrate caches expose this service.
/// </summary>
public class SubstrateTransferGrpcService : GrpcServiceBase
{
    private readonly StateMachine.ISubstrateTransferStateControl _transferControl;

    public SubstrateTransferGrpcService(
        ModuleInfo moduleInfo,
        StateMachine.ISubstrateTransferStateControl transferControl,
        ILogger<SubstrateTransferGrpcService> logger)
        : base(moduleInfo, logger)
    {
        _transferControl = transferControl ?? throw new ArgumentNullException(nameof(transferControl));
    }

    /// <summary>
    /// Prepares to load a substrate.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> PrepareLoadAsync(
        string substrateId, 
        string sourceLocation, 
        ServerCallContext context)
    {
        var result = await _transferControl.PrepareLoadAsync(substrateId, sourceLocation, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Prepare load failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Confirms substrate load success.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> ConfirmLoadSuccessAsync(ServerCallContext context)
    {
        var result = await _transferControl.ConfirmLoadSuccessAsync(context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Confirm load success failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Confirms substrate load failure.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> ConfirmLoadFailureAsync(
        string errorMessage, 
        ServerCallContext context)
    {
        var result = await _transferControl.ConfirmLoadFailureAsync(errorMessage, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Confirm load failure failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Prepares to unload a substrate.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> PrepareUnloadAsync(
        string substrateId,
        string destinationLocation, 
        ServerCallContext context)
    {
        var result = await _transferControl.PrepareUnloadAsync(substrateId, destinationLocation, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Prepare unload failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Confirms substrate unload success.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> ConfirmUnloadSuccessAsync(ServerCallContext context)
    {
        var result = await _transferControl.ConfirmUnloadSuccessAsync(context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Confirm unload success failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Confirms substrate unload failure.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> ConfirmUnloadFailureAsync(
        string errorMessage, 
        ServerCallContext context)
    {
        var result = await _transferControl.ConfirmUnloadFailureAsync(errorMessage, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Confirm unload failure failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Resets the transfer state to idle.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> ResetAsync(ServerCallContext context)
    {
        var result = await _transferControl.ResetAsync(context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Reset failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    private static RobotTransferState ToGrpcState(StateMachine.SubstrateTransferState state)
    {
        return state switch
        {
            StateMachine.SubstrateTransferState.Idle => RobotTransferState.RobotStateIdle,
            StateMachine.SubstrateTransferState.Loading => RobotTransferState.RobotStatePicking,
            StateMachine.SubstrateTransferState.LoadSuccess => RobotTransferState.RobotStateIdle,
            StateMachine.SubstrateTransferState.Unloading => RobotTransferState.RobotStatePlacing,
            StateMachine.SubstrateTransferState.UnloadSuccess => RobotTransferState.RobotStateIdle,
            StateMachine.SubstrateTransferState.Error => RobotTransferState.RobotStateError,
            _ => RobotTransferState.RobotStateUnspecified
        };
    }
}
