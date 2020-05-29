"""Setuptools script for Grapl graph-descriptions"""

import os
import shutil
import subprocess
import sys

from distutils.spawn import find_executable

from distutils.command.build import build as _build
from distutils.command.clean import clean as _clean

from setuptools.command.build_py import build_py as _build_py
from setuptools.command.develop import develop as _develop
from setuptools.command.egg_info import egg_info as _egg_info
from setuptools.command.install import install as _install
from setuptools.command.sdist import sdist as _sdist
from setuptools.command.test import test as _test
from setuptools import setup

from wheel.bdist_wheel import bdist_wheel as _bdist_wheel

HERE = os.path.abspath(os.path.dirname(__file__))
PROTO_DIR = os.path.join(HERE, 'proto')
OUT_DIR_NAME = 'grapl_graph_descriptions'
OUT_DIR = os.path.join(HERE, OUT_DIR_NAME)

def is_comment(line):
    """check whether a line is a comment"""
    return line.strip().startswith('#')


def find_requirements():
    with open(os.path.join(HERE, 'requirements.txt')) as requirements:
        return tuple(
            line.strip() for line in requirements if not is_comment(line)
        )


def find_version():
    with open(os.path.join(HERE, 'VERSION')) as version:
        return version.read().strip()
    raise Exception('Could not find graph-descriptions version')


def find_protoc():
    """find the protobuf compiler

    returns absolute path to protoc, raises an exception if absent
    """
    if 'PROTOC' in os.environ:
        return os.environ['PROTOC']
    protoc = find_executable('protoc')
    if protoc is None:
        raise Exception('Could not locate protoc')
    return protoc


def compile_protobuf(proto_file, proto_dir, out_dir):
    """compile the .proto file to a *_pb2.py file in out_dir"""
    protoc = find_protoc()
    sys.stdout.write(f'compiling protobuf {proto_file}\n')
    if subprocess.call([
            protoc,
            f'--proto_path={proto_dir}',
            f'--python_out={out_dir}',
            proto_file
    ]) != 0:
        raise Exception(f'Failed to compile protobuf {proto_file}')


def find_proto_files():
    """find all the .proto files"""
    for (base_path, _, files) in os.walk(PROTO_DIR):
        for file_name in files:
            if file_name.endswith('.proto'):
                yield os.path.join(base_path, file_name)


def compile_all_protobufs():
    """walk the proto directory tree and compile every .proto file"""
    if not os.path.exists(OUT_DIR):
        os.mkdir(OUT_DIR)
    for proto_file in find_proto_files():
        compile_protobuf(proto_file, PROTO_DIR, OUT_DIR)
    # build a package structure in the OUT_DIR
    for (base_path, _, files) in os.walk(OUT_DIR):
        if '__init__.py' not in files:
            with open(os.path.join(base_path, '__init__.py'), 'w') as init:
                init.write('# generated by grapl-graph-descriptions/setup.py\n')
                init.write('import sys\n')
                init.write('from pathlib import Path\n\n')
                init.write('HERE = str(Path(__file__).parent)\n')
                init.write('if HERE not in sys.path:\n')
                init.write('    sys.path.append(HERE)\n')


class bdist_wheel(_bdist_wheel):
    def run(self):
        compile_all_protobufs()
        _bdist_wheel.run(self)


class build(_build):
    def run(self):
        compile_all_protobufs()
        _build.run(self)


class build_py(_build_py):
    def run(self):
        compile_all_protobufs()
        _build_py.run(self)


class clean(_clean):
    def run(self):
        if os.path.exists(OUT_DIR):
            shutil.rmtree(OUT_DIR)
        build_dir = os.path.join(HERE, 'build')
        if os.path.exists(build_dir):
            shutil.rmtree(build_dir)
        dist_dir = os.path.join(HERE, 'dist')
        if os.path.exists(dist_dir):
            shutil.rmtree(dist_dir)
        egg_info_dir = os.path.join(HERE, 'grapl_graph_descriptions.egg-info')
        if os.path.exists(egg_info_dir):
            shutil.rmtree(egg_info_dir)
        _clean.run(self)


class develop(_develop):
    def run(self):
        compile_all_protobufs()
        _develop.run(self)


class egg_info(_egg_info):
    def run(self):
        compile_all_protobufs()
        _egg_info.run(self)


class install(_install):
    def run(self):
        compile_all_protobufs()
        _install.run(self)


class sdist(_sdist):
    def run(self):
        compile_all_protobufs()
        _sdist.run(self)


class test(_test):
    def run(self):
        compile_all_protobufs()
        _test.run(self)


__version__ = find_version()

setup(
    name='grapl_graph_descriptions',
    version=__version__,
    author='Grapl, Inc.',
    author_email='grapl.code@graplsecurity.com',
    url='https://github.com/grapl-security/grapl',
    description='Grapl protobuf definitions',
    packages=(OUT_DIR_NAME,),
    zip_safe=False,
    cmdclass={
        'bdist_wheel': bdist_wheel,
        'build': build,
        'build_py': build_py,
        'clean': clean,
        'develop': develop,
        'egg_info': egg_info,
        'install': install,
        'sdist': sdist,
        'test': test
    },
    install_requires=find_requirements(),
    setup_requires=('wheel',)
)
