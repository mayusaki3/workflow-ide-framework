# WFIDE default font asset

The Framework default font is provisioned locally and is not committed to the repository.

Expected file:

```text
assets/fonts/default/NotoSansCJK-Regular.ttc
```

Setup:

Windows:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup_fonts.ps1
```

Linux:
```bash
chmod +x ./scripts/setup_fonts.sh
./scripts/setup_fonts.sh
```

The current default is Noto Sans CJK Regular. The setup mechanism is promoted from the P0-1b EmbeddedFont validation result; technical-validation assets are no longer runtime dependencies of the Framework Sample.
