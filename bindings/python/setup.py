import os
import shutil
import sys
from pathlib import Path

from setuptools import Distribution, find_packages, setup
from setuptools.command.build_py import build_py as _build_py


def _platform_lib_name() -> str:
    if sys.platform.startswith("win"):
        return "motor_abi.dll"
    if sys.platform == "darwin":
        return "libmotor_abi.dylib"
    return "libmotor_abi.so"


def _platform_gateway_name() -> str:
    if sys.platform.startswith("win"):
        return "ws_gateway.exe"
    return "ws_gateway"


def _candidate_gateway_paths() -> list[Path]:
    here = Path(__file__).resolve()
    repo_root = here.parents[2]
    bin_name = _platform_gateway_name()
    candidates: list[Path] = []

    env = os.getenv("MOTORBRIDGE_WS_GATEWAY_BIN")
    if env:
        candidates.append(Path(env).expanduser())

    candidates.append(repo_root / "target" / "release" / bin_name)
    candidates.append(here.parent / "src" / "motorbridge" / "bin" / bin_name)
    return candidates


def _resolve_gateway_path() -> Path:
    for path in _candidate_gateway_paths():
        if path.exists():
            return path
    tried = "\n".join(f"- {path}" for path in _candidate_gateway_paths())
    raise RuntimeError(
        "Cannot locate ws_gateway binary for wheel build.\n"
        f"Tried:\n{tried}\n"
        "Build gateway first (`cargo build -p ws_gateway --release`) or set MOTORBRIDGE_WS_GATEWAY_BIN."
    )


def _candidate_abi_paths() -> list[Path]:
    here = Path(__file__).resolve()
    repo_root = here.parents[2]
    lib_name = _platform_lib_name()
    candidates: list[Path] = []

    env = os.getenv("MOTORBRIDGE_LIB")
    if env:
        candidates.append(Path(env).expanduser())

    candidates.append(repo_root / "target" / "release" / lib_name)
    candidates.append(here.parent / "src" / "motorbridge" / "lib" / lib_name)
    return candidates


def _resolve_abi_path() -> Path:
    for p in _candidate_abi_paths():
        if p.exists():
            return p
    tried = "\n".join(f"- {p}" for p in _candidate_abi_paths())
    raise RuntimeError(
        "Cannot locate motor_abi shared library for wheel build.\n"
        f"Tried:\n{tried}\n"
        "Build ABI first (`cargo build -p motor_abi --release`) or set MOTORBRIDGE_LIB."
    )


class BuildPyWithAbi(_build_py):
    def run(self):
        super().run()
        abi_src = _resolve_abi_path()
        dst_dir = Path(self.build_lib) / "motorbridge" / "lib"
        dst_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(abi_src, dst_dir / abi_src.name)

        gateway_src = _resolve_gateway_path()
        gateway_dir = Path(self.build_lib) / "motorbridge" / "bin"
        gateway_dir.mkdir(parents=True, exist_ok=True)
        gateway_dst = gateway_dir / gateway_src.name
        shutil.copy2(gateway_src, gateway_dst)
        try:
            gateway_dst.chmod(0o755)
        except OSError:
            # Windows may not honor POSIX mode bits; keep best effort.
            pass


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


setup(
    name="motorbridge",
    version="0.1.9",
    description="Python SDK for motorbridge Rust ABI",
    long_description=open("README.md", encoding="utf-8").read(),
    long_description_content_type="text/markdown",
    author="motorbridge contributors",
    license="MIT",
    python_requires=">=3.10",
    package_dir={"": "src"},
    packages=find_packages(where="src"),
    package_data={"motorbridge": ["lib/*", "bin/*"]},
    include_package_data=True,
    entry_points={
        "console_scripts": [
            "motorbridge-cli=motorbridge.cli:main",
            "motorbridge-gateway=motorbridge.gateway:main",
        ]
    },
    distclass=BinaryDistribution,
    cmdclass={"build_py": BuildPyWithAbi},
    zip_safe=False,
)
