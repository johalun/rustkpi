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

import os

import proc


CLOUD_STORAGE_BASE_URL = 'https://storage.googleapis.com/'
CLOUD_STORAGE_PATH = 'wasm-llvm/builds/'


def Upload(local, remote):
  """Upload file to Cloud Storage."""
  if not os.environ.get('BUILDBOT_BUILDERNAME'):
    return
  remote = CLOUD_STORAGE_PATH + remote
  proc.check_call(
      ['gsutil.py', 'cp', '-a', 'public-read', local, 'gs://' + remote])
  return CLOUD_STORAGE_BASE_URL + remote


def Copy(copy_from, copy_to):
  """Copy from one Cloud Storage file to another."""
  if not os.environ.get('BUILDBOT_BUILDERNAME'):
    return
  copy_from = CLOUD_STORAGE_PATH + copy_from
  copy_to = CLOUD_STORAGE_PATH + copy_to
  proc.check_call(
      ['gsutil.py', 'cp', '-a', 'public-read',
       'gs://' + copy_from, 'gs://' + copy_to])
  return CLOUD_STORAGE_BASE_URL + copy_to
