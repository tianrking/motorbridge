#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser(description='Generate winget/scoop/choco metadata from release zip')
    ap.add_argument('--tag', required=True, help='Release tag, e.g. v0.1.4')
    ap.add_argument('--repo', required=True, help='GitHub repo, e.g. tianrking/motorbridge')
    ap.add_argument('--zip', required=True, dest='zip_path', help='Path to windows motor_cli zip asset')
    ap.add_argument('--out', required=True, help='Output root (inside repository)')
    args = ap.parse_args()

    tag = args.tag
    version = tag[1:] if tag.startswith('v') else tag
    zip_path = Path(args.zip_path).resolve()
    out_root = Path(args.out).resolve()
    out_root.mkdir(parents=True, exist_ok=True)

    checksum = sha256_of(zip_path)
    asset_name = zip_path.name
    url = f'https://github.com/{args.repo}/releases/download/{tag}/{asset_name}'

    # Scoop
    scoop_dir = out_root / 'scoop'
    scoop_dir.mkdir(parents=True, exist_ok=True)
    scoop_manifest = {
        'version': version,
        'description': 'Unified multi-vendor CAN motor command line tool',
        'homepage': f'https://github.com/{args.repo}',
        'license': 'MIT',
        'url': url,
        'hash': checksum,
        'bin': 'bin/motor_cli.exe',
    }
    (scoop_dir / 'motor-cli.json').write_text(json.dumps(scoop_manifest, indent=2) + '\n', encoding='utf-8')

    # Winget manifests
    winget_root = out_root / 'winget' / 'manifests' / 't' / 'tianrking' / 'motor-cli' / version
    winget_root.mkdir(parents=True, exist_ok=True)

    (winget_root / 'Tianrking.MotorCli.yaml').write_text(
        f'''PackageIdentifier: Tianrking.MotorCli\nPackageVersion: {version}\nDefaultLocale: en-US\nManifestType: version\nManifestVersion: 1.6.0\n''',
        encoding='utf-8',
    )

    (winget_root / 'Tianrking.MotorCli.locale.en-US.yaml').write_text(
        f'''PackageIdentifier: Tianrking.MotorCli\nPackageVersion: {version}\nPackageLocale: en-US\nPublisher: tianrking\nPublisherUrl: https://github.com/{args.repo}\nPackageName: motor_cli\nShortDescription: Unified multi-vendor CAN motor command line tool\nLicense: MIT\nLicenseUrl: https://github.com/{args.repo}/blob/main/LICENSE\nManifestType: defaultLocale\nManifestVersion: 1.6.0\n''',
        encoding='utf-8',
    )

    (winget_root / 'Tianrking.MotorCli.installer.yaml').write_text(
        f'''PackageIdentifier: Tianrking.MotorCli\nPackageVersion: {version}\nInstallers:\n  - Architecture: x64\n    InstallerType: zip\n    InstallerUrl: {url}\n    InstallerSha256: {checksum.upper()}\n    NestedInstallerType: portable\n    NestedInstallerFiles:\n      - RelativeFilePath: bin/motor_cli.exe\n        PortableCommandAlias: motor_cli\nManifestType: installer\nManifestVersion: 1.6.0\n''',
        encoding='utf-8',
    )

    # Chocolatey package metadata (template only; publishing still requires chocolatey.org push)
    choco_root = out_root / 'choco'
    tools_dir = choco_root / 'tools'
    tools_dir.mkdir(parents=True, exist_ok=True)

    (choco_root / 'motor-cli.nuspec').write_text(
        f'''<?xml version="1.0"?>\n<package >\n  <metadata>\n    <id>motor-cli</id>\n    <version>{version}</version>\n    <title>motor_cli</title>\n    <authors>tianrking</authors>\n    <projectUrl>https://github.com/{args.repo}</projectUrl>\n    <licenseUrl>https://github.com/{args.repo}/blob/main/LICENSE</licenseUrl>\n    <requireLicenseAcceptance>false</requireLicenseAcceptance>\n    <description>Unified multi-vendor CAN motor command line tool.</description>\n    <tags>can motor robotics</tags>\n  </metadata>\n  <files>\n    <file src="tools\\**" target="tools" />\n  </files>\n</package>\n''',
        encoding='utf-8',
    )

    (tools_dir / 'chocolateyinstall.ps1').write_text(
        f'''$ErrorActionPreference = 'Stop'\n\n$packageName = 'motor-cli'\n$toolsDir    = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"\n$url64       = '{url}'\n$checksum64  = '{checksum}'\n\nInstall-ChocolateyZipPackage `\n  -PackageName $packageName `\n  -Url64bit $url64 `\n  -UnzipLocation $toolsDir `\n  -Checksum64 $checksum64 `\n  -ChecksumType64 'sha256'\n\n$exePath = Join-Path $toolsDir 'bin\\motor_cli.exe'\nInstall-ChocolateyPath (Split-Path $exePath -Parent) 'Machine'\n''',
        encoding='utf-8',
    )

    print(f'Generated manifests for {tag}')
    print(f'  URL: {url}')
    print(f'  SHA256: {checksum}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
