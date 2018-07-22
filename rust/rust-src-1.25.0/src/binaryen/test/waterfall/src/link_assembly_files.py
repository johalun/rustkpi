#! /usr/bin/env python

#   Copyright 2015 WebAssembly Community Group participants
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
  outname = basename + '.wast'
  return os.path.join(outdir, outname)


def link(infile, outfile, extras):
  """Create the command-line for a linker invocation."""
  linker = extras['linker']
  basename = os.path.basename(linker)
  commands = {
      's2wasm': [linker, '--allocate-stack', '1048576', '-o', outfile, infile],
      'wasm-as': [linker, '-o', outfile, infile],
  }
  return commands[basename]


def run(linker, files, fails, out):
  """Link all files."""
  assert os.path.isfile(linker), 'Cannot find linker at %s' % linker
  assert os.path.isdir(out), 'Cannot find outdir %s' % out
  assembly_files = glob.glob(files)
  assert len(assembly_files), 'No files found by %s' % files
  return testing.execute(
      tester=testing.Tester(
          command_ctor=link,
          outname_ctor=create_outname,
          outdir=out,
          extras={'linker': linker}),
      inputs=assembly_files,
      fails=fails)


def getargs():
  import argparse
  parser = argparse.ArgumentParser(description='Link .s files into .wast.')
  parser.add_argument('--linker', type=str, required=True,
                      help='Linker path')
  parser.add_argument('--files', type=str, required=True,
                      help='Glob pattern for .s files')
  parser.add_argument('--fails', type=str, required=True,
                      help='Expected failures')
  parser.add_argument('--out', type=str, required=True,
                      help='Output directory')
  return parser.parse_args()


if __name__ == '__main__':
  args = getargs()
  sys.exit(run(args.linker, args.files, args.fails, args.out))
