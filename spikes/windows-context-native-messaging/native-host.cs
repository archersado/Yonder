using System;
using System.IO;
using System.Text;
using System.Text.RegularExpressions;

public static class NativeHost {
  const int MaxBytes = 1024 * 1024;

  public static void Main() {
    Console.InputEncoding = Encoding.UTF8;
    Console.OutputEncoding = Encoding.UTF8;
    var input = Console.OpenStandardInput();
    var output = Console.OpenStandardOutput();
    while (true) {
      var header = ReadExactly(input, 4, true);
      if (header == null) return;
      int length = BitConverter.ToInt32(header, 0);
      if (length < 0 || length > MaxBytes) throw new InvalidDataException("消息长度非法");
      byte[] incoming = ReadExactly(input, length, false);
      string json = new UTF8Encoding(false, true).GetString(incoming).Trim();
      if (!json.StartsWith("{") || !json.EndsWith("}")) throw new InvalidDataException("消息必须是 JSON 对象");
      Match type = Regex.Match(json, "\\\"type\\\"\\s*:\\s*\\\"([a-z.]+)\\\"");
      File.AppendAllText(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "native-host-events.log"), DateTime.UtcNow.ToString("O") + " type=" + (type.Success ? type.Groups[1].Value : "unknown") + " bytes=" + length + Environment.NewLine);
      byte[] body = Encoding.UTF8.GetBytes("{\"ok\":true}");
      output.Write(BitConverter.GetBytes(body.Length), 0, 4);
      output.Write(body, 0, body.Length);
      output.Flush();
    }
  }

  static byte[] ReadExactly(Stream stream, int count, bool allowEof) {
    var buffer = new byte[count];
    int offset = 0;
    while (offset < count) {
      int read = stream.Read(buffer, offset, count - offset);
      if (read == 0) {
        if (allowEof && offset == 0) return null;
        throw new EndOfStreamException("消息帧不完整");
      }
      offset += read;
    }
    return buffer;
  }
}
