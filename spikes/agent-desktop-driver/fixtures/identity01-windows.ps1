# CU-S4 IDENTITY-01 controlled fixture. It never sends desktop input.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$source = @'
using System;
using System.Diagnostics;
using System.Drawing;
using System.Runtime.InteropServices;
using System.Windows.Forms;

public sealed class YonderIdentity01Fixture : ApplicationContext
{
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool SetPropW(IntPtr hwnd, string name, IntPtr value);

    private const string RoleProperty = "YonderCUIdentity01Role";
    private const string GenerationProperty = "YonderCUIdentity01Generation";
    private const string ClickCountProperty = "YonderCUIdentity01ClickCount";
    private const string ButtonId = "YonderIdentity01Button";
    private readonly Form target;
    private readonly Form decoy;
    private Form replacement;
    private readonly Timer timeout;
    private int openForms = 2;

    private YonderIdentity01Fixture()
    {
        target = MakeForm(1, 1, 100);
        decoy = MakeForm(2, 2, 450);
        target.FormClosed += OnFormClosed;
        decoy.FormClosed += OnFormClosed;
        target.Show();
        decoy.Show();
        Console.WriteLine("{\"schema_version\":\"yonder-identity01-fixture/v1\",\"pid\":"
            + Process.GetCurrentProcess().Id + ",\"target_window_id\":\"w-"
            + target.Handle.ToInt64() + "\",\"decoy_window_id\":\"w-"
            + decoy.Handle.ToInt64() + "\"}");
        Console.Out.Flush();
        timeout = new Timer();
        timeout.Interval = 180000;
        timeout.Tick += delegate { target.Close(); decoy.Close(); if (replacement != null) replacement.Close(); };
        timeout.Start();
        System.Threading.ThreadPool.QueueUserWorkItem(delegate {
            string command;
            while ((command = Console.ReadLine()) != null) {
                if (command == "replace") target.BeginInvoke((Action)ReplaceTarget);
                if (command == "replace-button") target.BeginInvoke((Action)ReplaceButton);
            }
        });
    }

    private Form MakeForm(int role, int generation, int x)
    {
        var form = new Form();
        form.Text = "Yonder Identity Fixture";
        form.Name = "YonderIdentity01Window";
        form.Size = new Size(300, 150);
        form.StartPosition = FormStartPosition.Manual;
        form.Location = new Point(x, 100);
        AddButton(form);
        IntPtr handle = form.Handle;
        Put(handle, RoleProperty, role);
        Put(handle, GenerationProperty, generation);
        Put(handle, ClickCountProperty, 1);
        return form;
    }

    private static void AddButton(Form form)
    {
        var button = new Button();
        button.Name = ButtonId;
        button.Text = "Run";
        button.AccessibleName = "Run";
        button.Size = new Size(100, 36);
        button.Location = new Point(90, 42);
        int clicks = 0;
        button.Click += delegate {
            clicks++;
            Put(form.Handle, ClickCountProperty, clicks + 1);
        };
        form.Controls.Add(button);
    }

    private static void Put(IntPtr hwnd, string name, int value)
    {
        if (!SetPropW(hwnd, name, new IntPtr(value)))
            throw new InvalidOperationException("fixture property setup failed");
    }

    private void OnFormClosed(object sender, FormClosedEventArgs args)
    {
        openForms--;
        if (openForms == 0) ExitThread();
    }

    private void ReplaceTarget()
    {
        target.Close();
        replacement = MakeForm(1, 3, 100);
        replacement.FormClosed += OnFormClosed;
        openForms++;
        replacement.Show();
        Console.WriteLine("{\"schema_version\":\"yonder-identity01-replaced/v1\",\"pid\":"
            + Process.GetCurrentProcess().Id + ",\"replacement_window_id\":\"w-"
            + replacement.Handle.ToInt64() + "\"}");
        Console.Out.Flush();
    }

    private void ReplaceButton()
    {
        Control oldButton = target.Controls[0];
        target.Controls.Remove(oldButton);
        oldButton.Dispose();
        AddButton(target);
        Console.WriteLine("{\"schema_version\":\"yonder-identity01-button-replaced/v1\",\"pid\":"
            + Process.GetCurrentProcess().Id + ",\"window_id\":\"w-"
            + target.Handle.ToInt64() + "\"}");
        Console.Out.Flush();
    }

    public static void Start()
    {
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        Application.Run(new YonderIdentity01Fixture());
    }
}
'@

Add-Type -TypeDefinition $source -ReferencedAssemblies System.Windows.Forms, System.Drawing
[YonderIdentity01Fixture]::Start()
