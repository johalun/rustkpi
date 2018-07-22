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

import difflib
import math
import multiprocessing
import os
import os.path
import resource
import sys

import proc


class Result:
  """Result from a single test that was run."""

  def __init__(self, test, success, output):
    self.test = test
    self.success = success
    self.output = output

  def __str__(self):
    return '%s %s%s%s' % ('SUCCEEDED' if self.success else 'FAILED',
                          self.test, '\n' if self.output else '', self.output)

  def __nonzero__(self):
    return self.success

  def __lt__(self, other):
    """Sort by test name so that the output files can be compared easily."""
    return self.test < other.test

  def similarity(self, other):
    """Compare output similarity, returning a float in the range [0,1]."""
    # Even quick_ratio is fairly slow on big inputs, capture just the start.
    max_size = 1024
    return difflib.SequenceMatcher(None, self.output[:max_size],
                                   other.output[:max_size]).quick_ratio()


class Tester(object):
  """Test runner."""

  def __init__(self, command_ctor, outname_ctor, outdir, extras):
    """Command-line constructor accepting input and output file names."""
    if outdir:
      assert os.path.isdir(outdir), 'Expected output directory %s' % outdir
    self.command_ctor = command_ctor
    self.outname_ctor = outname_ctor
    self.outdir = outdir
    self.extras = extras

  @staticmethod
  def setlimits():
    # Set maximum CPU time to 90 seconds in child process
    try:
      resource.setrlimit(resource.RLIMIT_CPU, (90, 90))
    except:
      pass

  def __call__(self, test_file):
    """Execute a single test."""
    basename = os.path.basename(test_file)
    outfile = self.outname_ctor(self.outdir, test_file) if self.outdir else ''
    try:
      output = proc.check_output(
          self.command_ctor(test_file, outfile, self.extras),
          stderr=proc.STDOUT, cwd=self.outdir or os.getcwd(),
          preexec_fn=Tester.setlimits)
      # Flush the logged command so buildbots don't think the script is dead.
      sys.stdout.flush()
      return Result(test=basename, success=True, output=output)
    except proc.CalledProcessError as e:
      return Result(test=basename, success=False, output=e.output)


def get_expected_failures(fails):
  """One failure per line, some whitespace, Python-style comments."""
  assert os.path.isfile(fails), 'Cannot find known failures at %s' % fails
  res = []
  with open(fails, 'r') as fails_file:
    res = fails_file.readlines()
  return sorted([r for r in [r.split('#')[0].strip() for r in res] if len(r)])


class TriangularArray:
  """Indexed with two commutable keys."""

  def __init__(self):
    self.arr = {}

  def canonicalize(self, key):
    return (min(key[0], key[1]), max(key[0], key[1]))

  def __getitem__(self, key):
    return self.arr[self.canonicalize(key)]

  def __setitem__(self, key, value):
    k = self.canonicalize(key)
    # Support single-insertion only, the intended usage would be a bug if there
    # were multiple insertions of the same key.
    assert k not in self.arr, 'Double insertion of key %s' % k
    self.arr[k] = value

  def __iter__(self):
    return self.arr.iteritems()


class SimilarityGroup:
  """Group of similar results."""
  def __init__(self, tests, similarities):
    self.tests = sorted(tests)
    self.similarities = [100. * s for s in similarities]
    self.average = (sum(self.similarities) / len(self.similarities)
                    if self.similarities else 0.)
    squared_diffs = [(s - self.average) ** 2 for s in self.similarities]
    self.stddev = (math.sqrt(sum(squared_diffs) / len(squared_diffs))
                   if self.similarities else 0.)


def similarity(results, cutoff):
  """List of lists of result test names with similar outputs."""
  similarities = TriangularArray()
  for x in range(0, len(results)):
    for y in range(x + 1, len(results)):
      rx = results[x]
      ry = results[y]
      similarities[(rx.test, ry.test)] = rx.similarity(ry)
  # A maximum clique would be better suited to group similarities, but this
  # silly traversal is simpler and seems to do the job pretty well.
  similar_groups = []
  worklist = set()
  for k, v in similarities:
    if v > cutoff:
      worklist.add(k[0])
      worklist.add(k[1])
  for result in results:
    test = result.test
    if test in worklist:
      worklist.remove(test)
      group_tests = [test]
      group_similarities = []
      for other_result in results:
        other_test = other_result.test
        if other_test in worklist:
          similar = similarities[(test, other_test)]
          if similar > cutoff:
            worklist.remove(other_test)
            group_tests.append(other_test)
            group_similarities.append(similar)
      if len(group_tests) > 1:
        # Some tests could have similar matches which were more similar to
        # other tests, leaving this group with a single entry.
        similar_groups.append(SimilarityGroup(tests=group_tests,
                                              similarities=group_similarities))
  assert len(worklist) == 0, 'Failed emptying worklist %s' % worklist
  # Put all the ungrouped tests into their own group.
  grouped = set()
  for group in similar_groups:
    for test in group.tests:
      grouped.add(test)
  uniques = list(set([r.test for r in results]) - grouped)
  if uniques:
    s = [similarities[(uniques[0], u)] for u in uniques[1:]]
    similar_groups.append(SimilarityGroup(tests=uniques, similarities=s))
  return similar_groups


def execute(tester, inputs, fails):
  """Execute tests in parallel, output results, return failure count."""
  input_expected_failures = get_expected_failures(fails) if fails else []
  pool = multiprocessing.Pool()
  sys.stdout.write('Executing tests.')
  results = sorted(pool.map(tester, inputs))
  pool.close()
  pool.join()
  sys.stdout.write('\nDone.')
  successes = [r for r in results if r]
  failures = [r for r in results if not r]
  if not fails:
    return failures
  expected_failures = [t for t in failures
                       if t.test in input_expected_failures]
  unexpected_failures = [t for t in failures
                         if t.test not in input_expected_failures]
  unexpected_successes = [t for t in successes
                          if t.test in input_expected_failures]
  sys.stdout.write('\nResults:\n')
  for result in results:
    sys.stdout.write(str(result) + '\n\n')
  cutoff = 0.9
  similar_expected_failures = similarity(expected_failures, cutoff)
  for s in similar_expected_failures:
    tests = ' '.join(s.tests)
    if s.average >= cutoff * 100.:
      sys.stdout.write(('\nSimilar expected failures, '
                        'average %s%% similarity with stddev %s: '
                        '%s\n') % (s.average, s.stddev, tests))
      sample = [f for f in expected_failures if f.test == s.tests[0]][0]
      sys.stdout.write('Sample failure: %s\n' % sample)
    else:
      sys.stdout.write(('\nUngrouped expected failures, '
                        'average %s%% similarity with stddev %s: '
                        '%s\n') % (s.average, s.stddev, tests))
  similar_unexpected_failures = similarity(unexpected_failures, cutoff)
  for s in similar_unexpected_failures:
    tests = ' '.join(s.tests)
    if s.average >= cutoff * 100.:
      sys.stdout.write(('\nSimilar unexpected failures, '
                        'average %s%% similarity with stddev %s: '
                        '%s\n') % (s.average, s.stddev, tests))
      sample = [f for f in unexpected_failures if f.test == s.tests[0]][0]
      sys.stdout.write('Sample failure: %s\n' % sample)
    else:
      sys.stdout.write(('\nUngrouped unexpected failures, '
                        'average %s%% similarity with stddev %s: '
                        '%s\n') % (s.average, s.stddev, tests))
  if expected_failures:
    sys.stdout.write('Expected failures:\n')
    for f in expected_failures:
      sys.stdout.write('\t%s\n' % f.test)
  if unexpected_failures:
    sys.stdout.write('Unexpected failures:\n')
    for f in unexpected_failures:
      sys.stdout.write('\t%s\n' % f.test)
  if unexpected_successes:
    sys.stdout.write('Unexpected successes:\n')
    for f in unexpected_successes:
      sys.stdout.write('\t%s\n' % f.test)
  sys.stdout.write(
      '\n'.join(['\n',
                 'Ran %s tests.' % len(results),
                 'Got %s successes.' % len(successes),
                 'Got %s failures.' % len(failures),
                 'Expected %s failures.' % len(input_expected_failures),
                 'Got %s expected failures in %s similarity groups.' % (
                     len(expected_failures),
                     len(similar_expected_failures)),
                 'Got %s unexpected failures in %s similarity groups.' % (
                     len(unexpected_failures),
                     len(similar_unexpected_failures)),
                 'Got %s unexpected successes.' % len(unexpected_successes),
                 '\n']))
  return len(unexpected_failures) + len(unexpected_successes)
