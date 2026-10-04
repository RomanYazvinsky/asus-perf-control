using GHelper.Helpers;
using System;
using System.IO;
using System.Management;
using System.Runtime.InteropServices;

#nullable enable
namespace GHelper;

internal class AsusSAIO : DeviceACPI
{
  private const string FILE_NAME = "\\\\.\\\\AsusSAIO";
  private const uint CONTROL_CODE = 2148540528;

  [DllImport("kernel32.dll", CharSet = (CharSet) 4, SetLastError = true)]
  private static extern bool SetDllDirectory(string lpPathName);

  [DllImport("AsusWinIO64.dll")]
  private static extern void InitializeWinIo();

  private static string? GetSAIOPath()
  {
    try
    {
      using (ManagementObjectSearcher managementObjectSearcher = new ManagementObjectSearcher("Select * from Win32_SystemDriver WHERE Name='AsusSAIO'"))
      {
        using (ManagementObjectCollection.ManagementObjectEnumerator enumerator = managementObjectSearcher.Get().GetEnumerator())
        {
          if (enumerator.MoveNext())
            return Path.GetDirectoryName(enumerator.Current["PathName"].ToString());
        }
      }
    }
    catch (Exception ex)
    {
      Logger.WriteLine(ex.Message);
    }
    return (string) null;
  }

  public AsusSAIO()
  {
    try
    {
      this.Connect("\\\\.\\\\AsusSAIO");
      Logger.WriteLine("Connected to SAIO");
    }
    catch (Exception ex1)
    {
      string saioPath = AsusSAIO.GetSAIOPath();
      Logger.WriteLine($"SAIO PATH: {saioPath} {ex1.Message}");
      try
      {
        ProcessHelper.RunAsAdmin();
        ProcessHelper.StartEnableService(nameof (AsusSAIO));
        Logger.WriteLine("Init SAIO");
        this.Connect("\\\\.\\\\AsusSAIO");
        Logger.WriteLine("Connected to SAIO");
      }
      catch (Exception ex2)
      {
        Logger.WriteLine($"Can't connect to SAIO: {saioPath} {ex2.Message}");
        if (!ProcessHelper.IsUserAdministrator() || ProcessHelper.IsRunningAsSystem())
          return;
        Logger.WriteLine("Trying to start app as System Process");
        ProcessHelper.RestartAsSystem();
      }
    }
  }

  protected byte CallMethod(byte[] args, byte output = 0, string? log = "SAIO")
  {
    byte[] lpInBuffer = new byte[16 /*0x10*/];
    byte[] lpOutBuffer = new byte[16 /*0x10*/];
    Array.Copy((Array) args, 0, (Array) lpInBuffer, 0, args.Length);
    lpInBuffer[10] = output;
    this.Control(2148540528U, lpInBuffer, lpOutBuffer);
    if (log != null)
      Logger.WriteLine($"{log}: {BitConverter.ToString(lpOutBuffer)}");
    return lpOutBuffer[11];
  }

  public int GetFanCount()
  {
    this.FanInit();
    return (int) this.CallMethod(new byte[4]
    {
      (byte) 221,
      (byte) 3,
      (byte) 2,
      (byte) 48 /*0x30*/
    }, (byte) 1);
  }

  public int GetCPUTemp()
  {
    return (int) this.CallMethod(new byte[3]
    {
      (byte) 221,
      (byte) 3,
      (byte) 2
    }, (byte) 1, (string) null);
  }

  public int GetFanSpeed(AsusFan fanId = AsusFan.CPU)
  {
    this.FanInit();
    this.FanSelect(fanId);
    return 256 /*0x0100*/ * (int) this.CallMethod(new byte[4]
    {
      (byte) 221,
      (byte) 3,
      (byte) 2,
      (byte) 52
    }, (byte) 1) + (int) this.CallMethod(new byte[4]
    {
      (byte) 221,
      (byte) 3,
      (byte) 2,
      (byte) 51
    }, (byte) 1);
  }

  public void FanInit()
  {
    int num = (int) this.CallMethod(new byte[3]
    {
      (byte) 187,
      (byte) 1,
      (byte) 80 /*0x50*/
    }, (byte) 1);
  }

  private void FanSelect(AsusFan fanId)
  {
    int num = (int) this.CallMethod(new byte[5]
    {
      (byte) 221,
      (byte) 3,
      (byte) 130,
      (byte) 50,
      (byte) fanId
    });
  }

  public void FanControl(bool manual = true)
  {
    this.FanInit();
    int num = (int) this.CallMethod(new byte[5]
    {
      (byte) 221,
      (byte) 3,
      (byte) 130,
      (byte) 49,
      (byte) manual
    });
  }

  public void FanSpeed(AsusFan fanId, float speed = 0.0f)
  {
    this.FanInit();
    this.FanSelect(fanId);
    speed = Math.Min(100f, Math.Max(0.0f, speed));
    int num = (int) this.CallMethod(new byte[5]
    {
      (byte) 221,
      (byte) 3,
      (byte) 130,
      (byte) 53,
      (byte) ((double) byte.MaxValue * (double) speed / 100.0)
    });
  }
}
