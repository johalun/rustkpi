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
  outname = basename + '.wasm'
  return os.path.join(outdir, outname)


def assemble(infile, outfile, extras):
  """Create the command-line for an assembler invocation."""
  assembler = extras['assembler']
  basename = os.path.basename(assembler)
  commands = {
      'sexpr-wasm': [extras['assembler'], infile, '-o', outfile]
  }
  return commands[basename]


def run(assembler, files, fails, out):
  """Assemble all files."""
  assert os.path.isfile(assembler), 'Cannot find assembler at %s' % assembler
  assert os.path.isdir(out), 'Cannot find outdir %s' % out
  assembler_files = glob.glob(files)
  assert len(assembler_files), 'No files found by %s' % files
  return testing.execute(
      tester=testing.Tester(
          command_ctor=assemble,
          outname_ctor=create_outname,
          outdir=out,
          extras={'assembler': assembler}),
      inputs=assembler_files,
      fails=fails)


def getargs():
  import argparse
  parser = argparse.ArgumentParser(
      description='Assemble .wast files into .wasm.')
  parser.add_argument('--assembler', type=str, required=True,
                      help='Assembler path')
  parser.add_argument('--files', type=str, required=True,
                      help='Glob pattern for .wast files')
  parser.add_argument('--fails', type=str, required=True,
                      help='Expected failures')
  parser.add_argument('--out', type=str, required=True,
                      help='Output directory')
  return parser.parse_args()


if __name__ == '__main__':
  args = getargs()
  sys.exit(run(args.assembler, args.files, args.fails, args.out))
