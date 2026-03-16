from setuptools import Distribution, find_packages, setup


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


setup(
    name="motorbridge",
    version="0.1.0",
    description="Python SDK for motorbridge Rust ABI",
    long_description=open("README.md", encoding="utf-8").read(),
    long_description_content_type="text/markdown",
    author="motorbridge contributors",
    license="MIT",
    python_requires=">=3.9",
    package_dir={"": "src"},
    packages=find_packages(where="src"),
    package_data={"motorbridge": ["lib/*"]},
    include_package_data=True,
    entry_points={"console_scripts": ["motorbridge-cli=motorbridge.cli:main"]},
    distclass=BinaryDistribution,
    zip_safe=False,
)
