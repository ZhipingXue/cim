using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// Base class for all gRPC services.
/// Provides common utilities: Ok/Error, module validation, state conversion.
/// </summary>
public abstract class GrpcServiceBase
{
    protected readonly ILogger Logger;
    protected readonly ModuleInfo ModuleInfo;

    protected GrpcServiceBase(ModuleInfo moduleInfo, ILogger logger)
    {
        ModuleInfo = moduleInfo ?? throw new ArgumentNullException(nameof(moduleInfo));
        Logger = logger ?? throw new ArgumentNullException(nameof(logger));
    }

    protected static global::Cim.Common.StatusCode Ok(string message = "Success")
        => new() { Code = 0, Message = message };

    protected static global::Cim.Common.StatusCode Error(int code, string message)
        => new() { Code = code, Message = message };

    protected bool ValidateModuleId(global::Cim.Common.ModuleId? requestModuleId)
    {
        if (requestModuleId == null)
        {
            Logger.LogWarning("Request missing module_id");
            return false;
        }

        var expectedId = ModuleInfo.QualifiedId;
        var requestId = $"{requestModuleId.EquipmentId}/{requestModuleId.ModuleId_}";

        if (!string.Equals(expectedId, requestId, StringComparison.OrdinalIgnoreCase))
        {
            Logger.LogWarning("Module ID mismatch: expected {ExpectedId}, got {RequestId}", expectedId, requestId);
            return false;
        }

        return true;
    }

    protected static global::Cim.Control.ModuleState ToGrpcModuleState(ModuleLifecycle.ModuleState state)
    {
        return state switch
        {
            ModuleLifecycle.ModuleState.Unspecified => global::Cim.Control.ModuleState.Unspecified,
            ModuleLifecycle.ModuleState.Offline => global::Cim.Control.ModuleState.Offline,
            ModuleLifecycle.ModuleState.Initializing => global::Cim.Control.ModuleState.Initializing,
            ModuleLifecycle.ModuleState.Online => global::Cim.Control.ModuleState.Online,
            ModuleLifecycle.ModuleState.Processing => global::Cim.Control.ModuleState.Processing,
            ModuleLifecycle.ModuleState.Error => global::Cim.Control.ModuleState.Error,
            _ => global::Cim.Control.ModuleState.Unspecified
        };
    }
}
