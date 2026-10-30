#!/usr/bin/env python3
"""Check the sparse crates.io index before attempting an immutable release."""
import json
import os
from pathlib import Path
import tomllib
from urllib.error import HTTPError
from urllib.request import urlopen


def needs_publish(name, version):
    """Only a missing crate/version permits publication; registry errors fail closed."""
    name = name.lower()
    if len(name) == 1:
        index_path = f'1/{name}'
    elif len(name) == 2:
        index_path = f'2/{name}'
    elif len(name) == 3:
        index_path = f'3/{name[0]}/{name}'
    else:
        index_path = f'{name[:2]}/{name[2:4]}/{name}'
    try:
        with urlopen(f'https://index.crates.io/{index_path}', timeout=30) as response:
            entries = [json.loads(line) for line in response if line.strip()]
    except HTTPError as error:
        if error.code == 404:
            return True
        raise
    if not entries:
        raise ValueError('Registry returned an empty crate index')
    return all(entry['vers'] != version for entry in entries)


def main():
    manifest = Path(__file__).resolve().parents[1] / 'Cargo.toml'
    package = tomllib.loads(manifest.read_text())['package']
    publish = needs_publish(package['name'], package['version'])
    print(f"{package['name']} {package['version']}: "
          f"{'ready to publish' if publish else 'already published; skipping'}")
    if output := os.environ.get('GITHUB_OUTPUT'):
        with open(output, 'a', encoding='utf-8') as stream:
            stream.write(f'publish={str(publish).lower()}\n')


if __name__ == '__main__':
    main()
