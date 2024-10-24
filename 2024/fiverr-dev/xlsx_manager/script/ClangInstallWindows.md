For writing Excel we are using **_xlsxwriter-rs_**, which has a dependency on clang/llvm. Mentioned here(https://github.com/informationsea/xlsxwriter-rs)
### 1: Set Execution Policy
```
1. Open PowerShell as Administrator
2. Set the Execution Policy

Command: Set-ExecutionPolicy Bypass -Scope Process -Force
```

### 2: Verify the Execution Policy Change
```
Command: Get-ExecutionPolicy -List

        Scope ExecutionPolicy
        ----- ---------------
MachinePolicy       Undefined
   UserPolicy       Undefined
      Process          Bypass
  CurrentUser      Restricted
 LocalMachine       Undefined
```

### 3: Run Your Script Again
```
Command:  .\install_llvm_clang_choco.ps1
```