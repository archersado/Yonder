using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;

public static class OperationHook {
  const int WhKeyboardLl = 13;
  const int WhMouseLl = 14;
  const uint WmQuit = 0x0012;
  static long keyboardEvents;
  static long mouseEvents;
  static HookProc keyboardProc = CountKeyboard;
  static HookProc mouseProc = CountMouse;

  public static void Main(string[] args) {
    int seconds = args.Length == 1 ? int.Parse(args[0]) : 10;
    if (seconds < 1 || seconds > 60) throw new ArgumentOutOfRangeException("seconds");
    Console.WriteLine("Press Enter to start. Only event counts are recorded.");
    Console.ReadLine();
    IntPtr keyboard = SetWindowsHookEx(WhKeyboardLl, keyboardProc, IntPtr.Zero, 0);
    IntPtr mouse = SetWindowsHookEx(WhMouseLl, mouseProc, IntPtr.Zero, 0);
    if (keyboard == IntPtr.Zero || mouse == IntPtr.Zero) throw new InvalidOperationException("SetWindowsHookEx failed");
    uint thread = GetCurrentThreadId();
    using (var timer = new Timer(_ => PostThreadMessage(thread, WmQuit, IntPtr.Zero, IntPtr.Zero), null, seconds * 1000, Timeout.Infinite)) {
      try {
        MSG message;
        while (GetMessage(out message, IntPtr.Zero, 0, 0) > 0) {
          TranslateMessage(ref message);
          DispatchMessage(ref message);
        }
      } finally {
        UnhookWindowsHookEx(keyboard);
        UnhookWindowsHookEx(mouse);
      }
    }
    string json = "{\"recording\":false,\"keyboard_events\":" + keyboardEvents + ",\"mouse_events\":" + mouseEvents + ",\"hooks_released\":true}";
    File.WriteAllText(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "operation-hook-evidence.json"), json);
    Console.WriteLine(json);
  }

  static IntPtr CountKeyboard(int code, IntPtr message, IntPtr data) {
    if (code >= 0) Interlocked.Increment(ref keyboardEvents);
    return CallNextHookEx(IntPtr.Zero, code, message, data);
  }

  static IntPtr CountMouse(int code, IntPtr message, IntPtr data) {
    if (code >= 0) Interlocked.Increment(ref mouseEvents);
    return CallNextHookEx(IntPtr.Zero, code, message, data);
  }

  delegate IntPtr HookProc(int code, IntPtr message, IntPtr data);
  [StructLayout(LayoutKind.Sequential)] struct MSG { public IntPtr hwnd; public uint message; public UIntPtr wParam; public IntPtr lParam; public uint time; public int x; public int y; }
  [DllImport("user32.dll")] static extern IntPtr SetWindowsHookEx(int id, HookProc callback, IntPtr module, uint threadId);
  [DllImport("user32.dll")] static extern bool UnhookWindowsHookEx(IntPtr hook);
  [DllImport("user32.dll")] static extern IntPtr CallNextHookEx(IntPtr hook, int code, IntPtr message, IntPtr data);
  [DllImport("user32.dll")] static extern int GetMessage(out MSG message, IntPtr window, uint min, uint max);
  [DllImport("user32.dll")] static extern bool TranslateMessage(ref MSG message);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessage(ref MSG message);
  [DllImport("user32.dll")] static extern bool PostThreadMessage(uint threadId, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
}
