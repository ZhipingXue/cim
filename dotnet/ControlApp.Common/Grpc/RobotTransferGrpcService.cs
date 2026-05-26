using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// gRPC service for robot transfer state control.
/// Only robot modules expose this service.
/// </summary>
public class RobotTransferGrpcService : GrpcServiceBase
{
    private readonly StateMachine.IRobotTransferStateControl _transferControl;

    public RobotTransferGrpcService(
        ModuleInfo moduleInfo,
        StateMachine.IRobotTransferStateControl transferControl,
        ILogger<RobotTransferGrpcService> logger)
        : base(moduleInfo, logger)
    {
        _transferControl = transferControl ?? throw new ArgumentNullException(nameof(transferControl));
    }

    /// <summary>
    /// Commands the robot to pick from a source location.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> PickAsync(
        string sourceLocation, 
        ServerCallContext context)
    {
        var result = await _transferControl.PickAsync(sourceLocation, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Pick failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Commands the robot to place at a destination location.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> PlaceAsync(
        string destinationLocation, 
        ServerCallContext context)
    {
        var result = await _transferControl.PlaceAsync(destinationLocation, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Place failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Commands the robot to move to a destination location.
    /// </summary>
    public async Task<SetRobotTransferStateResponse> MoveAsync(
        string destinationLocation, 
        ServerCallContext context)
    {
        var result = await _transferControl.MoveAsync(destinationLocation, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Move failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    /// <summary>
    /// Sets the robot transfer state directly (for simple transitions).
    /// </summary>
    public async Task<SetRobotTransferStateResponse> SetStateAsync(
        RobotTransferState state, 
        ServerCallContext context)
    {
        var targetState = FromGrpcState(state);
        var result = await _transferControl.TryTransitionToAsync(targetState, context.CancellationToken);

        return new SetRobotTransferStateResponse
        {
            Status = result.IsSuccess 
                ? Ok() 
                : Error((int)result.ErrorCode, result.ErrorMessage ?? "Transition failed"),
            State = ToGrpcState(_transferControl.CurrentState)
        };
    }

    private static StateMachine.RobotTransferState FromGrpcState(RobotTransferState state)
    {
        return state switch
        {
            RobotTransferState.RobotStateIdle => StateMachine.RobotTransferState.Idle,
            RobotTransferState.RobotStateMoving => StateMachine.RobotTransferState.Moving,
            RobotTransferState.RobotStatePicking => StateMachine.RobotTransferState.Picking,
            RobotTransferState.RobotStatePlacing => StateMachine.RobotTransferState.Placing,
            RobotTransferState.RobotStateAligning => StateMachine.RobotTransferState.Aligning,
            RobotTransferState.RobotStateError => StateMachine.RobotTransferState.Error,
            _ => StateMachine.RobotTransferState.Idle
        };
    }

    private static RobotTransferState ToGrpcState(StateMachine.RobotTransferState state)
    {
        return state switch
        {
            StateMachine.RobotTransferState.Idle => RobotTransferState.RobotStateIdle,
            StateMachine.RobotTransferState.Moving => RobotTransferState.RobotStateMoving,
            StateMachine.RobotTransferState.Picking => RobotTransferState.RobotStatePicking,
            StateMachine.RobotTransferState.Placing => RobotTransferState.RobotStatePlacing,
            StateMachine.RobotTransferState.Aligning => RobotTransferState.RobotStateAligning,
            StateMachine.RobotTransferState.Error => RobotTransferState.RobotStateError,
            _ => RobotTransferState.RobotStateUnspecified
        };
    }
}
