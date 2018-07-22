#! /usr/bin/env python

#   Copyright 2016 WebAssembly Community Group participants
#
#   Licensed under the Apache License, Version 2.0 (the "License");
#   you may not use this file except in compliance with the License.
#   You may obtain a copy of the License at
#
#       http://www.apache.org/licenses/LICENSE-2.0
#
#   Unless required by applicable law or agreed to in writing, software
#   distributed under the License is distributed on an "AS IS" BASIS,
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#   See the License for the specific language governing permissions and
#   limitations under the License.

import glob
import os
import os.path
import sys

import testing


def create_outname(outdir, infile):
  """Create the output file's name."""
  basename = os.path.basename(infile)
  outname = basename + '.out'
  return os.path.join(outdir, outname)


def execute(infile, outfile, extras):
  """Create the command-line for an execution."""
  runner = extras['runner']
  basename = os.path.basename(runner)
  out_opt = ['-o', outfile] if outfile else []
  extra_files = extras['extra_files']
  config = basename
  wasmjs = [extras['wasmjs']] if extras['wasmjs'] else []
  if basename == 'd8':
    config = basename + ('-wasm' if wasmjs else '-asm2wasm')
  commands = {
      'wasm-shell': [runner, '--entry=main', infile] + out_opt,
      'd8-wasm': [runner, '--expose-wasm'] + wasmjs + [
          '--', infile] + extra_files,
      'd8-asm2wasm': [runner, '--expose-wasm', infile],
      'wasm.opt': [runner, infile]
  }
  return commands[config]


def run(runner, files, fails, out, wasmjs='', extra_files=[]):
  """Execute all files."""
  assert os.path.isfile(runner), 'Cannot find runner at %s' % runner
  if out:
    assert os.path.isdir(out), 'Cannot find outdir %s' % out
  if wasmjs:
    assert os.path.isfile(wasmjs), 'Cannot find wasm.js %s' % wasmjs
  executable_files = glob.glob(files)
  assert len(executable_files), 'No files found by %s' % files
  return testing.execute(
      tester=testing.Tester(
          command_ctor=execute,
          outname_ctor=create_outname,
          outdir=out,
          extras={
              'runner': runner,
              'wasmjs': wasmjs,
              'extra_files': extra_files if extra_files else []
          }),
      inputs=executable_files,
      fails=fails)


def getargs():
  import argparse
  parser = argparse.ArgumentParser(description='Execute .wast or .wasm files.')
  parser.add_argument('--runner', type=str, required=True,
                      help='Runner path')
  parser.add_argument('--files', type=str, required=True,
                      help='Glob pattern for .wast / .wasm files')
  parser.add_argument('--fails', type=str, required=True,
                      help='Expected failures')
  parser.add_argument('--out', type=str, required=False,
                      help='Output directory')
  parser.add_argument('--wasmjs', type=str, required=False,
                      help='JavaScript support runtime for WebAssembly')
  parser.add_argument('--extra', type=str, required=False, action='append',
                      help='Extra files to pass to the runner')
  return parser.parse_args()


if __name__ == '__main__':
  args = getargs()
  sys.exit(run(args.runner, args.files, args.fails, args.out, args.wasmjs,
               args.extra))
