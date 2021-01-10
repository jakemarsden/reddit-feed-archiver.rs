# Reddit feed archiver

Only configurable at compile-time at the moment, by editing `src/main.rs` and rebuilding.

Find your account's token by logging into Reddit and visiting https://old.reddit.com/prefs/feeds/.
Choose any of the _RSS_ or _JSON_ links and copy its target location. The token is the value of
the `?feed=` URI attribute.
