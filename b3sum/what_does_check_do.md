# How does `b3sum --check` behave exactly?<br>or: Are filepaths text?

Most of the time, `b3sum --check` is a drop-in replacement for `md5sum --check`
and other Coreutils hashing tools. It consumes a checkfile (the output of a
regular `b3sum` command), re-hashes all the files listed there, and returns
success if all of those hashes are still correct. What makes this more
complicated than it might seem, is that representing filepaths as text means we
need to consider many possible edge cases of unrepresentable filepaths. This
document describes all of these edge cases in detail.

## The simple case

Here's the result of running `b3sum a b c/d` in a directory that contains
those three files:

```
0b8b60248fad7ac6dfac221b7e01a8b91c772421a15b387dd1fb2d6a94aee438  a
6ae4a57bbba24f79c461d30bcb4db973b9427d9207877e34d2d74528daa84115  b
2d477356c962e54784f1c5dc5297718d92087006f6ee96b08aeaf7f3cd252377  c/d
```

If we pipe that output into `b3sum --check`, it will exit with status zero
(success) and print:

```
a: OK
b: OK
c/d: OK
```

If we delete `b` and change the contents of `c/d`, and then use the same
checkfile as above, `b3sum --check` will exit with a non-zero status (failure)
and print:

```
a: OK
b: FAILED (No such file or directory (os error 2))
c/d: FAILED
```

In these typical cases, `b3sum` and `md5sum` have identical output for success
and very similar output for failure.

## Escaping newlines and backslashes

Since the checkfile format (the regular output format of `b3sum`) is
newline-separated text, we need to worry about what happens when a filepath
contains newlines, or worse. Suppose we create a file named `abc[newline]def`
(7 characters). One way to create such a file is with a Python one-liner like
this:

```
open("abc\ndef", "w")
```

Here's what we see if we run e.g. `b3sum *` to hash that file:

```
\af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262  abc\ndef
```

Notice two things. First, `b3sum` put a single `\` character at the front of
the line. This indicates that the filepath contains escape sequences that
`b3sum --check` will need to unescape. Then, `b3sum` replaced the newline
character in the filepath with the two-character escape sequence `\n`.
Similarly, if the filepath contained a backslash, `b3sum` would escape it as
`\\` in the output. So far, all of this behavior is still identical to
`md5sum`.

## Invalid Unicode

This is where `b3sum` and `md5um` start to diverge. Apart from the newline and
backslash escapes noted in the previous section, `md5sum` copies all other
filepath bytes verbatim to its output. That means its output is "ASCII plus
whatever bytes we got from the command line". This creates two problems:

1. Working with text that's not UTF-8 is kind of gross.
2. Windows support.

What's the problem with Windows? To start with, there's a fundamental
difference in how Unix and Windows represent filepaths. Unix filepaths are
"usually UTF-8" and Windows filepaths are "usually UTF-16". That means that a
file named `abc` is typically represented as the bytes `[97, 98, 99]` on Unix
and as the bytes `[97, 0, 98, 0, 99, 0]` on Windows. We don't want to "just
copy the bytes", because among other reasons we want a checkfile created on one
machine to be meaningful on other machines, for example if it's committed to a
git repo or hosted on the web. Instead, the natural thing to do is to parse
platform-specific filepath bytes into the Unicode characters they represent,
and then to write them out in some consistent encoding. (In practice we're
going to choose UTF-8, but for the purposes of this discussion it doesn't
matter what we choose.)

[TODO]
