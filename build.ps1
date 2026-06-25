# ═══════════════════════════════════════════════════════
# Minagi VTF Baker —— 构建打包脚本
# ═══════════════════════════════════════════════════════

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# ── Step 0: 读取版本号 ──
Write-Host "🌸 桃华的打包脚本启动啦~" -ForegroundColor Magenta
Write-Host ""

$TauriConfPath = "$ScriptDir\src-tauri\tauri.conf.json"
if (-not (Test-Path $TauriConfPath)) {
    Write-Host "❌ 找不到 tauri.conf.json" -ForegroundColor Red
    exit 1
}

$TauriConf = Get-Content $TauriConfPath -Raw | ConvertFrom-Json
$Version = $TauriConf.version
$ProductName = $TauriConf.productName

Write-Host "📦 产品：$ProductName" -ForegroundColor Cyan
Write-Host "🔖 版本：$Version" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: 执行 Cargo Tauri Build ──
Write-Host "🔨 Step 1/3: 编译打包..." -ForegroundColor Yellow
Write-Host ""

Push-Location "$ScriptDir\src-tauri"
try {
    cargo tauri build
    if ($LASTEXITCODE -ne 0) {
        throw "cargo tauri build 返回错误码 $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

# ── Step 2: 复制到 dist/ ──
Write-Host "📋 Step 2/3: 复制安装包到 dist/ ..." -ForegroundColor Yellow

$DistDir = "$ScriptDir\dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
}

$NsisDir = "$ScriptDir\src-tauri\target\release\bundle\nsis"
$SourceInstaller = "$NsisDir\${ProductName}_${Version}_x64-setup.exe"

if (-not (Test-Path $SourceInstaller)) {
    Write-Host "❌ 找不到安装包，搜索中..." -ForegroundColor Red
    Get-ChildItem -Path "$ScriptDir\src-tauri\target" -Recurse -Filter "*setup*.exe" -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "   找到: $($_.FullName)" -ForegroundColor Gray
    }
    exit 1
}

$TargetName = "${ProductName}_v${Version}_setup.exe"
Copy-Item -Path $SourceInstaller -Destination "$DistDir\$TargetName" -Force

# 也复制便携版
$ExeSource = "$ScriptDir\src-tauri\target\release\minagi-vtf-baker.exe"
if (Test-Path $ExeSource) {
    $PortableName = "${ProductName}_v${Version}_portable.exe"
    Copy-Item -Path $ExeSource -Destination "$DistDir\$PortableName" -Force
    Write-Host "  ✓ 便携版已复制" -ForegroundColor Green
}

$FileSize = [math]::Round((Get-Item "$DistDir\$TargetName").Length / 1MB, 2)
Write-Host "  ✓ 安装包已复制" -ForegroundColor Green
Write-Host ""

# ── Step 3: 打印结果 ──
Write-Host "✨ Step 3/3: 打包完成！" -ForegroundColor Magenta
Write-Host ""
Write-Host "  📁 安装包: $DistDir\$TargetName" -ForegroundColor White
Write-Host "  📏 大小:    $FileSize MB" -ForegroundColor White
Write-Host "  🔖 版本:    $Version" -ForegroundColor White
Write-Host ""
Write-Host "🌸 桃华的报告：打包完成~" -ForegroundColor Magenta
