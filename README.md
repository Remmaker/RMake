# RMake Reborn

A fast, readable, **no-bullshit** build system for personal C/C++ projects.

No CMakeLists hell. No 500-line Makefiles.  
Just a clean `.rm` file you can read (and edit) in 30 seconds.

### Example · conf.rm
```ini
#   <- Comment token, no multiline token, just use line by line commentary.
//  <- Also comment
;   <- Also also comment

build 
{
    compiler=g++
    flags=-std=c++17 -Wall -Werror -Wextra -ggdb
    src=src/*.cpp
    include=src/include
    lpaths=My/lib/path/lib
    lflags=mylib opengl32
    target=out.out
}

run
{
    interpreter=python
    prescript=pretest.py
    run=progtest.exe
    postscript=testpost.py
}
```

##### See `example/`

# Usage 
```text
rmake                    # build (auto-finds *.rm)
rmake mygame.rm | mygame # build a specific file
rmake run                # run the [run] section (no rebuild)
rmake help               # this message
```
