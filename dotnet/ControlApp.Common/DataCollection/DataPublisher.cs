using System.Text.Json;
using Microsoft.Extensions.Logging;

namespace Cim.ControlApp.Common.DataCollection;

/// <summary>
/// Publishes data to the Data Collection service (Zenohd).
/// </summary>
public interface IDataPublisher
{
    Task PublishAsync(string key, string value, CancellationToken ct = default);
    Task PublishAsync<T>(string key, T value, CancellationToken ct = default) where T : notnull;
}

/// <summary>
/// Data publisher implementation that publishes to Zenohd via HTTP/gRPC.
/// For now, uses in-memory store; will connect to Zenohd when available.
/// </summary>
public class DataPublisher : IDataPublisher
{
    private readonly string _moduleId;
    private readonly ILogger<DataPublisher>? _logger;

    public DataPublisher(string moduleId, ILogger<DataPublisher>? logger = null)
    {
        _moduleId = moduleId;
        _logger = logger;
    }

    public Task PublishAsync(string key, string value, CancellationToken ct = default)
    {
        var fullKey = $"equipment/{_moduleId}/{key}";
        _logger?.LogDebug("Publishing {Key} = {Value}", fullKey, value);

        // TODO: Publish to Zenohd when available
        return Task.CompletedTask;
    }

    public Task PublishAsync<T>(string key, T value, CancellationToken ct = default) where T : notnull
    {
        var json = JsonSerializer.Serialize(value);
        return PublishAsync(key, json, ct);
    }
}
