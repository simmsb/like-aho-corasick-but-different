# like-aho-corasick-but-different
Something for finding all word boundary delimited substrings in another string, and carrying extra info on match states.

Mainly derived from [aho-corasick](https://github.com/BurntSushi/aho-corasick),
but with only has what's needed in, has extra logic for only matching
strings that are delimited by word boundaries, and allows for attaching data to
match states to be added to returned matches.
