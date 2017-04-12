#!/bin/python

import collections
import os
import re
import subprocess
import time

GHOSTLY_PATH = '/usr/bin/ghostly'

ALLIE_DBG = '../target/debug/allie'

# Old versions
ALLIE_0_8 = './bin/allie_v0.8'
ALLIE_0_7 = './bin/allie_v0.7'
ALLIE_0_6 = './bin/allie_v0.6'
ALLIE_0_5 = './bin/allie_v0.5'
ALLIE_0_4 = './bin/allie_v0.4'
ALLIE_0_3 = './bin/allie_v0.3'
ALLIE_0_2 = './bin/allie_v0.2'
ALLIE_0_1 = './bin/allie_v0.1'

RESULT_RE = re.compile(r'^name:(?P<name>[^;]+);wins:(?P<wins>\d+);score:(?P<score>\d+)$')

ROUNDS = 25

Score = collections.namedtuple('Score', ['wins', 'score'])

def parse_result(server_output):
    ret = {}

    for result in server_output.decode("utf-8").split('\n'):
        match = RESULT_RE.match(result)
        if match is not None:
            ret[match.group('name')] = Score(int(match.group('wins')), int(match.group('score')))

    return ret

def benchmark():
    # Start the server
    server = subprocess.Popen([GHOSTLY_PATH
                            #    , '--headless'
                               , '--start-at', '2'
                               , '--tickless'
                               , '--rounds', str(ROUNDS)]
                              , stdout=subprocess.PIPE
                              , stderr=subprocess.PIPE)
    time.sleep(1)

    # Start the bots, ignoring any output
    devnull = open(os.devnull, 'w')
    subprocess.Popen([ALLIE_0_8], stdout=devnull, stderr=devnull)
    subprocess.Popen([ALLIE_DBG])

    # Wait here until the match is finished
    out, _ = server.communicate()

    # Parse the result
    results = parse_result(out)
    total_wins = sum(t.wins for t in results.values())
    total_score = sum(t.score for t in results.values())

    # Print the result
    for name, result in results.items():
        print(name + ":")
        print('\tWins: {}/{} {:.2f}%'
              .format(result.wins
                      , total_wins
                      , result.wins / total_wins * 100 if total_wins > 0 else 0))
        print('\tScore: {}/{} {:.2f}%'
              .format(result.score
                      , total_score
                      , result.score / total_score * 100 if total_score > 0 else 0))

if __name__ == '__main__':
    benchmark()
