import pathlib

import yaml

from wok import config
from wok.config import schema


def test_001_config_1_0_load(data_dir: pathlib.Path) -> None:
    conf = config.Config.load(data_dir / '101_wok.yml')

    conf_data = schema.config.dump(conf)

    assert conf_data == {
        'version': '1.0',
        'repos': [
            {'path': 'prj-1', 'url': 'repos/prj-1', 'ref': 'master'},
            {'path': 'prj-2', 'url': 'repos/prj-2', 'ref': 'master'},
        ],
    }


def test_002_config_1_0_save(data_dir: pathlib.Path, tmp_path: pathlib.Path) -> None:
    conf: config.Config = schema.config.load(
        {
            'version': '1.0',
            'repos': [
                {'path': 'prj-1', 'url': 'repos/prj-1', 'ref': 'master'},
                {'path': 'prj-2', 'url': 'repos/prj-2', 'ref': 'master'},
            ],
        }
    )
    conf._path = tmp_path / 'wok.yml'

    conf.save()

    assert yaml.safe_load(conf._path.open()) == yaml.safe_load(
        data_dir.joinpath('101_wok.yml').open()
    )
