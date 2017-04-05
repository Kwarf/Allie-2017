# Allie benchmarking utility
Intended for testing to determine improvements/regressions by running multiple
versions against eachother.

## Usage details
When saving results below, use information as returned by
`date +"%Y-%m-%d %H:%M" && git rev-parse --short HEAD` for time and versioning.

# Results
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
