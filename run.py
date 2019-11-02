#!/bin/python3
import subprocess
import sys

example_args = ['--example', 'generator', r'--features=opengl-rendering,vector-rendering']

if len(sys.argv) == 1:
    subprocess.run(["cargo", "run"] + example_args + ['--release'])
elif len(sys.argv) == 2:
    if sys.argv[1] == "flamegraph":
        subprocess.run(["cargo", "flamegraph"] + example_args)
    else:
        subprocess.run(["cargo", "run"] + example_args + ['--release', '--'] + [sys.argv[1]])
elif len(sys.argv) == 3:
    if sys.argv[1] == "flamegraph":
        subprocess.run(["cargo", "flamegraph"] + example_args + ['--'] + [sys.argv[2]])
    else:
        raise Exception("invalid combination of arguments")
