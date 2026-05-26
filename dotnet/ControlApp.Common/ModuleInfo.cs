using Microsoft.Extensions.Configuration;

namespace Cim.ControlApp.Common;

/// <summary>
/// Module information loaded from configuration.
/// Uniquely identifies a module in the equipment cluster.
/// </summary>
public class ModuleInfo
{
    /// <summary>
    /// Unique module identifier (GUID).
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Module name (e.g., "ProcessChamber", "SubstrateCache").
    /// </summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Module index in a multi-module cluster (0-based).
    /// Available when multiple identical modules exist in equipment.
    /// </summary>
    public int? Index { get; set; }

    /// <summary>
    /// Equipment identifier this module belongs to.
    /// </summary>
    public string EquipmentId { get; set; } = string.Empty;

    /// <summary>
    /// Module type discriminator (chamber, robot, loadport, subcache).
    /// </summary>
    public string ModuleType { get; set; } = string.Empty;

    /// <summary>
    /// Gets the display name including index if available.
    /// </summary>
    public string DisplayName => Index.HasValue ? $"{Name}-{Index}" : Name;

    /// <summary>
    /// Gets the full qualified module identifier.
    /// </summary>
    public string QualifiedId => $"{EquipmentId}/{DisplayName}";

    /// <summary>
    /// Loads ModuleInfo from IConfiguration section.
    /// </summary>
    public static ModuleInfo FromConfiguration(IConfiguration configuration)
    {
        var moduleInfo = new ModuleInfo
        {
            Id = Guid.TryParse(configuration["module:id"], out var id) ? id : Guid.NewGuid(),
            Name = configuration["module:name"] ?? "Unknown",
            EquipmentId = configuration["module:equipment_id"] ?? configuration["equipment_id"] ?? "eq-01",
            ModuleType = configuration["module:type"] ?? "unknown"
        };

        if (int.TryParse(configuration["module:index"], out var index))
        {
            moduleInfo.Index = index;
        }

        return moduleInfo;
    }

    /// <summary>
    /// Creates a default ModuleInfo for fallback scenarios.
    /// </summary>
    public static ModuleInfo CreateDefault(string moduleType, string name, int? index = null)
    {
        return new ModuleInfo
        {
            Id = Guid.NewGuid(),
            Name = name,
            ModuleType = moduleType,
            Index = index,
            EquipmentId = "eq-01"
        };
    }
}
