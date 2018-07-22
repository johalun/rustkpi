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

# This module is intended to be a drop-in replacement for the standard
# subprocess module, with the difference that it logs commands before it runs
# them. Everything not overriden should pass through to the subprocess module
# via the import trick below.

# Imports subprocess in its own namespace so we can always refer directly to
# its attributes.
import subprocess
import os
import sys
# Imports all of subprocess into the current namespace, effectively
# re-exporting everything.
from subprocess import * # flake8: noqa


# Now we can override any parts of subprocess we want, while leaving the rest.
def check_call(cmd, **kwargs):
  cwd = kwargs.get('cwd', os.getcwd())
  c = ' '.join('"' + c + '"' if ' ' in c else c for c in cmd)
  print 'subprocess.check_call(`%s`, cwd=`%s`)' % (c, cwd)
  sys.stdout.flush()
  subprocess.check_call(cmd, **kwargs)
  sys.stdout.flush()


def check_output(cmd, **kwargs):
  cwd = kwargs.get('cwd', os.getcwd())
  c = ' '.join('"' + c + '"' if ' ' in c else c for c in cmd)
  print 'subprocess.check_output(`%s`, cwd=`%s`)' % (c, cwd)
  sys.stdout.flush()
  try:
    return subprocess.check_output(cmd, **kwargs)
  finally:
    sys.stdout.flush()
