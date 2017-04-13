# Allie benchmarking utility
Intended for testing to determine improvements/regressions by running multiple
versions against eachother.

## Usage details
When saving results below, use information as returned by
`date +"%Y-%m-%d %H:%M" && git rev-parse --short HEAD` for time and versioning.

# Results
## 2017-04-13 21:36
Note that this version can occationally get stuck when in PickPellets, going back
and forth between the same two positions, I should investigate why. It's fairly
bad if it happens.
```
Allie (v1.1):
        Wins: 10/25 40.00%
        Score: 2592/5632 46.02%
Allie (v0.9):
        Wins: 15/25 60.00%
        Score: 3040/5632 53.98%
```
```
Allie (v1.1):
        Wins: 21/25 84.00%
        Score: 3054/5656 54.00%
Allie (v0.7):
        Wins: 4/25 16.00%
        Score: 2602/5656 46.00%
```
```
Allie (v1.1):
        Wins: 20/26 76.92%
        Score: 2614/4759 54.93%
Allie (v0.5):
        Wins: 6/26 23.08%
        Score: 2145/4759 45.07%
```

## 2017-04-13 14:05
Went back to only considering the strategy with the highest priority.
Considering all and applying weights in a bunch of directions got messy, it was
hard to see what was happening and why.

Gets in deadlocks in spawn when run against v0.8, since it's not possible to get
out without getting "too close" to other bots.
```
Allie (v0.9):
        Wins: 12/26 46.15%
        Score: 2820/5703 49.45%
Allie (v0.7):
        Wins: 14/26 53.85%
        Score: 2883/5703 50.55%
```
```
Allie (v0.9):
        Wins: 15/25 60.00%
        Score: 3461/5961 58.06%
Allie (v0.5):
        Wins: 10/25 40.00%
        Score: 2500/5961 41.94%
```
## 2017-04-12 21:23
Too many changes to list.. It seems much better than 0.7, but still loses to
0.5.
```
Allie (0.8):
        Wins: 100/100 100.00%
        Score: 16071/21716 74.01%
Allie (0.7):
        Wins: 0/100 0.00%
        Score: 5645/21716 25.99%
```
```
Allie (0.8):
        Wins: 7/27 25.93%
        Score: 2292/5314 43.13%
Allie (0.5):
        Wins: 20/27 74.07%
        Score: 3022/5314 56.87%
```

## 2017-04-01 02:52
There's been a ton of changes from the previous benchmark run. Mainly in
pathfinding performance. I also fixed an issue that prevented the bot from
being able to walk through tunnels.

However it often loses against the old version, this seems to be mostly due to
entering risky dead ends and getting trapped and killed. This will hopefully
be resolved in the next version when I implement dead-end detection.
```
Allie (v0.7):
        Wins: 34/100 34.00%
        Score: 146/274 53.28%
Allie (v0.5):
        Wins: 66/100 66.00%
        Score: 128/274 46.72%
```

## 2017-03-28 22:19
Now has three strategies, other than the previous pellet-picking one there's:
- __Avoidance__ activates when an enemy is standing on a tile right next to us,
and will move away from them, but not if we have super pellet powers and they
don't.
- __Hunter__ activates when picking up a super pellet, and hunts any other bots
that do not have pellet powers themself.
```
Allie (v0.5):
        Wins: 100/100 100.00%
        Score: 157/274 57.30%
Allie (v0.4):
        Wins: 0/100 0.00%
        Score: 117/274 42.70%
```

## 2017-03-27 19:38
Some very minor improvements, mostly a combination of the previous two versions.
The matches were very deterministic, the new version won because it got to the
super pellet first, and then they collided.
```
Allie (v0.4):
        Wins: 100/100 100.00%
        Score: 197/197 100.00%
Allie (v0.3):
        Wins: 0/100 0.00%
        Score: 0/197 0.00%
```

## 2017-03-27 04:43
The bot is now using BFS to locate the closest pellet and moving there.
While there's pellets next to it it will just keep on going for them, with no
pathfinding. It will primarily keep walking in the same direction if profitable.

What's interesting is that this lowered the win rate vs. the previous version.
It's interesting to look at them, this new BFS version acts more like a vaccum,
picking up stray points that the old one left behind, it will probably be best
to implement a combination of the two strategies.
```
Allie (v0.3):
        Wins: 42/100 42.00%
        Score: 130/274 47.45%
Allie (v0.2):
        Wins: 58/100 58.00%
        Score: 144/274 52.55%
```

## 2017-03-26 06:28
The bot can now move in a somewhat sensible way, resulting in a 100% win rate
against the random one.

The score not being summed (as I wrote below) is a known bug in the server.
It only shows the last round.
```
Allie (v0.2):
        Wins: 101/101 100.00%
        Score: 185/185 100.00%
Allie (v0.1):
        Wins: 0/101 0.00%
        Score: 0/185 0.00%
```

## 2017-03-23 22:58
First version that learned how to walk.. Randomly.. It felt fair to test it
against rand0m.py. Score seems low for 100 turns for some reason, and number of
wins adds up to more than 100 somehow.
```
Allie (v0.1):
        Wins: 58/108 53.70%
        Score: 13/15 86.67%
rand0m.py:
        Wins: 50/108 46.30%
        Score: 2/15 13.33%
```
