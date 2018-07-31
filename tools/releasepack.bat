REM @echo off
pushd %~dp0
cd ../lazebird.rabbit.rabbit/bin/Release/
echo %cd%
set pwd=%cd%
..\..\..\tools\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe /out:../../../release/sRabbit.exe lazebird.rabbit.rabbit.exe /wildcards *.dll zh-CN/lazebird.rabbit.rabbit.resources.dll
popd
REM pause
