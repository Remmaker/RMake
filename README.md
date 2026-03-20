# RMake 

A fast, readable, **no-bullshit** build system for personal C/C++ projects.

No CMakeLists hell. No 500-line Makefiles.  
Just a clean `.rm` file you can read (and edit) in 30 seconds.

### Example · conf.rm
```ini
#   <- Comment token, no multiline token, just use line by line commentary.
//  <- Also comment
;   <- Also also comment

myvar=myexe.out

build 
{
    compiler=g++
    flags=-std=c++17 -Wall -Werror -Wextra -ggdb
    src=src/*.cpp
    include=src/include
    lpaths=My/lib/path/lib
    lflags=mylib opengl32
    target=${myvar}
}

run
{
    target=${myvar}
    rebuild=t | f | true | false
}
```

##### See `example/`

# Usage 
```text
rmake                    # execute build section (auto-find first .rm file)
rmake mygame.rm | mygame # execute build section on a specific file
rmake run                # execute run section
rmake help               # this message
```
