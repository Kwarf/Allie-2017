# Allie benchmarking utility
Intended for testing to determine improvements/regressions by running multiple
versions against eachother.

## Usage details
When saving results below, use information as returned by
`date +"%Y-%m-%d %H:%M" && git rev-parse --short HEAD` for time and versioning.

# Results
## 2017-03-26 06:28
The bot can now move in a somewhat sensible way, resulting in a 100% win rate
against the random one.

The score not being summed (as I wrote below) is a known bug in the server.
It only shows the last round.
```
Allie (65143ae):
        Wins: 101/101 100.00%
        Score: 185/185 100.00%
Allie (c657773):
        Wins: 0/101 0.00%
        Score: 0/185 0.00%
```

## 2017-03-23 22:58
First version that learned how to walk.. Randomly.. It felt fair to test it
against rand0m.py. Score seems low for 100 turns for some reason, and number of
wins adds up to more than 100 somehow.
```
Allie (c657773):
        Wins: 58/108 53.70%
        Score: 13/15 86.67%
rand0m.py:
        Wins: 50/108 46.30%
        Score: 2/15 13.33%
```
