# FunDSP player

Player for FunDSP creations.

```
FunDSP player

Usage: fundsp_player <BUILD> <COMMAND>

Commands:
  live   Play to audio device
  dummy  Play to 'devnull'
  file   Play to wave file
  help   Print this message or the help of the given subcommand(s)

Arguments:
  <BUILD>  DSP build to play

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
Play to audio device

Usage: fundsp_player <BUILD> live [OPTIONS]

Options:
  -d, --device <DEVICE>  Audio device - if not found a list is displayed
  -h, --help             Print help
```

```
Play to 'devnull'

Usage: fundsp_player <BUILD> dummy [OPTIONS]

Options:
  -s, --seconds <SECONDS>  Seconds to generate [default: 10]
  -h, --help               Print help
```

```
Play to wave file

Usage: fundsp_player <BUILD> file [OPTIONS]

Options:
  -s, --seconds <SECONDS>    Seconds to generate [default: 10]
  -f, --filename <FILENAME>  Output file [default: output.wav]
  -h, --help                 Print help
```

Currently it assumes that you have checked out fundsp to "../fundsp", as I use
it for funDSP development.

## TODO

- Implement file-command
- Implement some interaction

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in fundsp_player by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
