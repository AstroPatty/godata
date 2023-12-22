import toml
from packaging import version

module_path = "godata/__init__.py"
with open(module_path) as f:
    for line in f:
        if line.startswith("__version__"):
            client_version_attribute = line.split("=")[1].strip().strip('"')
            break

client_version_attribute = version.parse(client_version_attribute)


cargo_toml = toml.load("Cargo.toml")
server_version = version.parse(cargo_toml["package"]["version"])

pypi_toml = toml.load("pyproject.toml")
client_version_toml = version.parse(pypi_toml["tool"]["poetry"]["version"])

if client_version_toml != server_version:
    raise ValueError(
        f"Client version {client_version_toml} "
        f"does not match server version {server_version}"
    )

if client_version_toml != client_version_attribute:
    raise ValueError(
        f"Client version {client_version_toml} does not match client attribute version"
        f" {client_version_attribute}"
    )
