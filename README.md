# Problem

Implement a command-line parser program to to list all file and folder inside the provided zip file.
The parser should print the following fields for each file and folder, one file per line.

* Name
* Whether the item is a folder
* Uncompressed Size
* Modified date
* Comment

The parser should be extensible for future additions, such that another developer could easily add
support for encryption and file content extraction, or add other fields present in the same data
structures to be parsed and output.
You do not need to support the following in your parser: encryption; file content extraction; ZIP64.

Do not use an existing built-in or third-party library, such as Python’s zipfile module or Rust’s
zip crate.

Here are two resources for Zip file format documentation:

* https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT
* https://users.cs.jmu.edu/buchhofp/forensics/formats/pkzip-printable.html

There are numerous other resources on the Internet documenting the format, as well.

```text
Code_sample$ ./zip_parser exercise.zip
folder True 0 2022-05-19T10:51:38
folder/folder True 0 2022-05-19T10:51:18 A nested folder
folder/folder/test.txt False 125 2022-05-19T10:56:30
...
Code_sample$
```

## FAQ

1. Which language should I use?

    * Our primary languages are Python, C++, Rust, and TypeScript, and we have some C#, Java, and
JavaScript here and there. Please pick whichever one of these you feel most comfortable with.

2. How long should this take?
    * Please contact us if it seems like this will take longer than four hours of total work.
We ask that you aim to complete the exercise in one week, though please contact us if you need more
time.

3. How should I deliver the code?

    * Emailing a zip or tar archive is fine, or a link to a public github repository.
Do not email unzipped code files or executables, as they will often not make it through our email
filters.

4. Should the code run and work?

    * Yes. Platforms and language runtimes vary, so please let us know how to get the parser and its
tests (if any) to run, and we’ll endeavor to get it to work on our own systems.

5. Is this a trick question?

    * No. There’s room for interpretation and tradeoffs, but there’s no exact right approach we’re
looking for or some “gotcha” shortcut.

# Solution

This solution is in Rust; testing and building it depend on Rust edition 2021.
Please visit <https://rustup.rs/> for instructions on installing Rust on your system.

This crate should be functional across Windows, macOS, and Linux; but it has only been tested on
Debian Linux 11 so far.
Once the library and/or command line utility are built and installed on a similar (enough) system,
they will run without needing either Rust or this source code repository; however this usage is
currently outside the scope of this exercise (Hint: build the release executable, then copy it from
`./target/release/zp` to another machine).

Source code is organized in a single git repository / workspace with a library crate and a command
line utility.
This enables direct use of the library separate from the command line utility.

There are just a few external dependencies:

* [binrw](https://crates.io/crates/binrw): declarative type-based parser/combinator for binary data
* [assert\_cmd](https://crates.io/crates/assert_cmd): integration tests
* [clap](https://crates.io/crates/clap): command line argument parser
* [hex](https://crates.io/crates/hex): handling binary data

The first version of this solution (0.1.0) was a straightforward approach in Rust, but I believed a
third-party crate meant for the task would be a better overall solution because while it "costs"
more in terms of time and effort to learn it and then depending on it going forward, it enables
using a common API and should handle generic errors and some potential security issues.

Initial research revealed several crates that might work:

* [binread](https://crates.io/crates/binread)
    * [binread: Declarative Rust Binary Parsing](https://jam1.re/blog/binread-a-declarative-rust-binary-parsing-library)
    * *Outdated; use binrw instead*
* [binrw](https://crates.io/crates/binrw)
* [byteorder](https://crates.io/crates/byteorder)
* [nom](https://crates.io/crates/nom)
* [pest](https://crates.io/crates/pest)
    * *Better for grammars?*
* [serde](https://crates.io/crates/serde)

The most well-designed one in my opinion was binread.
Unfortunately, it seems to not have anywhere near the following of nom, pest, or serde.
I'd used serde extensively but had never written a custom parser for it.

I made an attempt using binread, but ran into an issue where it was erroring at the end of the input
file instead of just stopping parsing (reported via
[binread#47](https://github.com/jam1garner/binread/issues/47)).
In researching possible fixes, I found binrw, which looked like where the development effort had
shifted.
I tried replacing binread with binrw, and it was a very clean swap, but ran into a much
nicer-looking but effectively the same error.
I reported it via [binrw#125](https://github.com/jam1garner/binrw/issues/125).
Fortunately, a committer (Colin Snover) took notice and gave a hint at the root cause, so I
implemented it, confirmed it fixed the issue in my
[proof-of-concept](https://github.com/qtfkwk/br-eof-poc) and binrw-based
version of zp, and created a [pull request](https://github.com/jam1garner/binrw/pull/126) to get the
fix in binrw.
In the meantime, zp can use my fork of binrw.

As of version 0.2.0, zp has been redesigned to use binrw.
It maintains the same function API via `process` and `process_file` functions, but adds a struct API
via `Zip`, which enables reading a file once and producing the output separately.

## Check out the repository

```bash
git clone https://github.com/qtfkwk/zp.git
cd zp
```

## Run tests

Tests include:

* Unit tests on `lib`
* Integration tests on `cli`

```bash
cargo test
```

## Build/run debug executable

```bash
cargo build
```

## Build/run release executable

```bash
cargo build --release
```

## Install

This command builds the release executable and installs it to `~/.cargo/bin` which should be in your
`PATH`.

```bash
cargo install --path cli
```

## View usage

```text
$ zp -h
zp 0.2.1
Zip Parser

USAGE:
    zp [OPTIONS] [FILES]...

ARGS:
    <FILES>...    One or more zip files

OPTIONS:
    -h, --help       Print help information
    -v               Verbosity
    -V, --version    Print version information
```

## Run against `exercise.zip`

```text
$ zp exercise.zip
folder00/	true	0	2022-05-19T10:51:38	
folder00/folder00-00/	true	0	2022-05-19T10:51:18	A nested folder
folder00/folder00-00/test00-00-00.txt	false	4	2020-08-25T09:05:38	
folder00/folder00-00/test00-00-01.txt	false	125	2022-05-19T10:56:30	
folder00/folder00-00/test00-00-02.txt	false	4	2020-08-25T09:05:38	
folder00/test00-00.txt	false	95	2022-05-19T10:57:24	
folder00/test00-01.txt	false	0	2021-08-25T13:04:38	This file doesn't have any content
folder01/	true	0	2022-05-19T10:51:26	
folder01/exercise.zip	false	2272	2022-05-19T11:05:08	
folder01/test01-00.txt	false	127	2022-05-19T10:53:46	This is a comment
test00.txt	false	4	2020-08-25T09:05:38	A top level file
test01.txt	false	4	2020-08-25T09:05:38	
test02.txt	false	4	2020-08-25T09:05:38	

```

## Run against `exercise.zip` (verbose mode)

```text
$ zp -v exercise.zip
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x5673 ((10, 51, 38))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0009 (9)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f" ("folder00/")
extra_field = ""
file_data = ""
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x5669 ((10, 51, 18))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0015 (21)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f666f6c64657230302d30302f" ("folde\
r00/folder00-00/")
extra_field = ""
file_data = ""
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x0025 (37)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f666f6c64657230302d30302f746573743\
0302d30302d30302e747874" ("folder00/folder00-00/test00-00-00.txt\
")
extra_field = ""
file_data = "74657374"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x570f ((10, 56, 30))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xa2fb7922 (2734389538)
compressed_size = 0x00000071 (113)
uncompressed_size = 0x0000007d (125)
file_name_length = 0x0025 (37)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f666f6c64657230302d30302f746573743\
0302d30302d30312e747874" ("folder00/folder00-00/test00-00-01.txt\
")
extra_field = ""
file_data = "1dccbb0d02311004d056a6012c213a4084040434b098019ff07\
94fdee54e646e03099a73257cc297bcde5efb61a62126295762d19acf1670a04\
e9988526032135396423768c552196ff0c431e098c47b7bda8fdf4287c8802df\
f3cd11c7a41ba8f52067f84dede5861bdf1849d46d7fa01"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x0025 (37)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f666f6c64657230302d30302f746573743\
0302d30302d30322e747874" ("folder00/folder00-00/test00-00-02.txt\
")
extra_field = ""
file_data = "74657374"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x572c ((10, 57, 24))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xbacf92e2 (3134165730)
compressed_size = 0x0000005b (91)
uncompressed_size = 0x0000005f (95)
file_name_length = 0x0016 (22)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f7465737430302d30302e747874" ("fol\
der00/test00-00.txt")
extra_field = ""
file_data = "0dcac10d40401005d0bb440fbf014e4a50820658c34ec28cec8\
e88dbb621a1b9adc49edfcbe91d3c05cae98910c5a12c06164cc4b2620d7a09c\
e03bce0d6136e9432ad289ce76de6e8117527e39d629bd387069d79f4ea4c435\
dfd"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x6893 ((13, 4, 38))
mod_date = 0x5319 ((2021, 8, 25))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0016 (22)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230302f7465737430302d30312e747874" ("fol\
der00/test00-01.txt")
extra_field = ""
file_data = ""
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x566d ((10, 51, 26))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0009 (9)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230312f" ("folder01/")
extra_field = ""
file_data = ""
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x58a4 ((11, 5, 8))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xd23d584b (3527235659)
compressed_size = 0x000003c1 (961)
uncompressed_size = 0x000008e0 (2272)
file_name_length = 0x0015 (21)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230312f65786572636973652e7a6970" ("folde\
r01/exercise.zip")
extra_field = ""
file_data = "a5940b504c5118c7ef6d1fd3a69d966884b885f228ba3b0c434\
6c96b9264cb630ad9b5976dda47ec3276518648e53143435192c1a4b1e3d1786\
b24378cd79647a19945e8610cd5344a4dc339f7eeeedddaaed6b88ff9ceb9f39\
def77fee77edf17bd88c31d8cc04bbbbc2416b1bb04e05daf51ca894dc1c1931\
9b7c45e6e43ecddac8320cb1237caa564a1f752f794d4375c3086ef58b6253a4\
2ab0323ea99a4dbaa83732bd915f1585112eba7ef3abd11cc7638190687617c9\
ede16bae0226ee5f27368a0ef8cd0342e77caa51cb4a059bfc4fc254aaee6088\
e6bc76e2f3f76c7bca64b7aced77ffc0a59d5719357f2a9a8dc7d92b562cf98a\
373a6292a24de9a6612ff90436edbf6eed0fb39e98f7606fd0ea91a6108bb754\
8c24f4d339b574a4b5bd2f2e7d774a2ff275dec283d1048af3b6cba150f6609e\
0f5b285b15f0ad7091fdf1386868a7895b7e77adc45a322a277f157de8f2adff\
fede0beda6bbe85378de4c9f74f9a7c42788f9b32441585817caf48f28a4c681\
03ce7dcf950eda9ce161bc7e535a8ea1b076e0ef8942f3bf1329d9faf6ffb1a1\
9beba9b517544e11d833097e3766c47cfe48d8a3dbdf0c98cd2ebc06d42bbb84\
30966a93d43e37468dca23428eb89b03c58c48bd7f342cfb495ed9929bc90712\
0c858526d584a3e9b95bca77e95e0b336af2a52533071dcee7a0139a074c251d\
2bd415a159e7936af5878a37b7c448b9bc0f54c4399e9d7e5f386b87663c1ab1\
8f389ecacccd10fd38de19a84b69f0fa6c791f3befb7772d87f28fc4a2bb6fe3\
5277cf17ff0b5cb06d46536c256ab632c23116d98dfe18660d40714198ac4fff\
091e424bd41adf651cac3ab8d866a940ecd52df20b4872d74409ff9db0bd331d\
56b010c6fb5a58392ca20260c53031d841ca317d254963ab10aa2c2c6395135d\
4f9dbef425a849a205561ca0881bbd85d445b1adb474b71c0163a81c57b63734\
8573417606a5d4751f6edf04c0aebbcda6568ff58319bda80240a475906cbd24\
57a60f350c722763cd579eb3e625056b6672b65eb6a642f9814eaa33d58387e3\
6ce4417478ee3310e28aa28aecdad41750f6e0c839cb5e7371c3e066cac22518\
bad4f5412985c4368d5013a4c21dd426052b51e5ba751eb08758f3a51fdb54e1\
2989de03d12b8f2decd08787856bbe6ca459211c9d2a840e88136917b5dd8da9\
63d4795157f12c6af97c95ba10d3b88be061c5a2478a440934a65d3c4d229005\
86403fb73e8bed17f3d448f24fc2828a6d324634a620ba1a40eb63f169331122\
b0b6761d9925f289e5b0de7cec74eb1c4ee3fd3398b3382e94ce7f1a18f3bb83\
11e821473e0ec0f"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x56b7 ((10, 53, 46))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xf832f72a (4164089642)
compressed_size = 0x0000006c (108)
uncompressed_size = 0x0000007f (127)
file_name_length = 0x0016 (22)
extra_field_length = 0x0000 (0)
file_name = "666f6c64657230312f7465737430312d30302e747874" ("fol\
der01/test01-00.txt")
extra_field = ""
file_data = "2d8bcb0dc23010055b790540a4f4c0853b0dac898d2dadb3d67\
a51c4cd3d7085e65c09e4739cd14c6f9f2b2883e609c40bbd2a96c40ce761d14\
38aa59caa0db8fd294af10a0908a4e7c0cff9b1a97a5af7ad9fd453de9b948b8\
a23c787ad436f5ff4f6c6385ac445ee26fa03"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x000a (10)
extra_field_length = 0x0000 (0)
file_name = "7465737430302e747874" ("test00.txt")
extra_field = ""
file_data = "74657374"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x000a (10)
extra_field_length = 0x0000 (0)
file_name = "7465737430312e747874" ("test01.txt")
extra_field = ""
file_data = "74657374"
data_descriptor = None
---
sig = 0x504b0304 (Local file header)
version = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x000a (10)
extra_field_length = 0x0000 (0)
file_name = "7465737430322e747874" ("test02.txt")
extra_field = ""
file_data = "74657374"
data_descriptor = None
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x5673 ((10, 51, 38))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0009 (9)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000010 (16)
lfh_offset = 0x00000000 (0)
file_name = "666f6c64657230302f" ("folder00/")
extra_field = "0a00200000000000010018005bef1d52986bd8015bef1d529\
86bd801c87ec7b5e87ad601"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x5669 ((10, 51, 18))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0015 (21)
extra_field_length = 0x0024 (36)
file_comment_length = 0x000f (15)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000010 (16)
lfh_offset = 0x00000027 (39)
file_name = "666f6c64657230302f666f6c64657230302d30302f" ("folde\
r00/folder00-00/")
extra_field = "0a0020000000000001001800f8351647986bd801f83516479\
86bd801bd126bc0e87ad601"
file_comment = "41206e657374656420666f6c646572" ("A nested folde\
r")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x0025 (37)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x0000005a (90)
file_name = "666f6c64657230302f666f6c64657230302d30302f746573743\
0302d30302d30302e747874" ("folder00/folder00-00/test00-00-00.txt\
")
extra_field = "0a002000000000000100180061a801cfe87ad60168cf893c9\
86bd80183a8893c986bd801"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x570f ((10, 56, 30))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xa2fb7922 (2734389538)
compressed_size = 0x00000071 (113)
uncompressed_size = 0x0000007d (125)
file_name_length = 0x0025 (37)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x000000a1 (161)
file_name = "666f6c64657230302f666f6c64657230302d30302f746573743\
0302d30302d30312e747874" ("folder00/folder00-00/test00-00-01.txt\
")
extra_field = "0a002000000000000100180098c40801996bd801db081f019\
96bd801d91b8a3c986bd801"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x0025 (37)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x00000155 (341)
file_name = "666f6c64657230302f666f6c64657230302d30302f746573743\
0302d30302d30322e747874" ("folder00/folder00-00/test00-00-02.txt\
")
extra_field = "0a002000000000000100180061a801cfe87ad601276b8a3c9\
86bd801276b8a3c986bd801"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x572c ((10, 57, 24))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xbacf92e2 (3134165730)
compressed_size = 0x0000005b (91)
uncompressed_size = 0x0000005f (95)
file_name_length = 0x0016 (22)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x0000019c (412)
file_name = "666f6c64657230302f7465737430302d30302e747874" ("fol\
der00/test00-00.txt")
extra_field = "0a00200000000000010018004563e120996bd8019413f2209\
96bd801e2d762d2e87ad601"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x6893 ((13, 4, 38))
mod_date = 0x5319 ((2021, 8, 25))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0016 (22)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0022 (34)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x0000022b (555)
file_name = "666f6c64657230302f7465737430302d30312e747874" ("fol\
der00/test00-01.txt")
extra_field = "0a00200000000000010018000ba8c5aadb99d70174c6b81a9\
96bd80160a96792976bd801"
file_comment = "546869732066696c6520646f65736e277420686176652061\
6e7920636f6e74656e74" ("This file doesn't have any content")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x566d ((10, 51, 26))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0x00000000 (0)
compressed_size = 0x00000000 (0)
uncompressed_size = 0x00000000 (0)
file_name_length = 0x0009 (9)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000010 (16)
lfh_offset = 0x0000025f (607)
file_name = "666f6c64657230312f" ("folder01/")
extra_field = "0a0020000000000001001800d0c1b94a986bd801d0c1b94a9\
86bd8015eb2afc4e87ad601"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x58a4 ((11, 5, 8))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xd23d584b (3527235659)
compressed_size = 0x000003c1 (961)
uncompressed_size = 0x000008e0 (2272)
file_name_length = 0x0015 (21)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x00000286 (646)
file_name = "666f6c64657230312f65786572636973652e7a6970" ("folde\
r01/exercise.zip")
extra_field = "0a0020000000000001001800055175359a6bd80183ed87409\
a6bd80100d535409a6bd801"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x0014 (20)
flags = 0x0000 (0)
compression = 0x0008 (8)
mod_time = 0x56b7 ((10, 53, 46))
mod_date = 0x54b3 ((2022, 5, 19))
crc32 = 0xf832f72a (4164089642)
compressed_size = 0x0000006c (108)
uncompressed_size = 0x0000007f (127)
file_name_length = 0x0016 (22)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0011 (17)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x0000067a (1658)
file_name = "666f6c64657230312f7465737430312d30302e747874" ("fol\
der01/test01-00.txt")
extra_field = "0a00200000000000010018006d8b5b9e986bd801e66264f29\
86bd801418e01d5e87ad601"
file_comment = "54686973206973206120636f6d6d656e74" ("This is a \
comment")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x000a (10)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0010 (16)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x0000071a (1818)
file_name = "7465737430302e747874" ("test00.txt")
extra_field = "0a002000000000000100180061a801cfe87ad60168cf893c9\
86bd801501e6522986bd801"
file_comment = "4120746f70206c6576656c2066696c65" ("A top level \
file")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x000a (10)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x00000746 (1862)
file_name = "7465737430312e747874" ("test01.txt")
extra_field = "0a002000000000000100180061a801cfe87ad601d91b8a3c9\
86bd8010d3244d6e87ad601"
file_comment = "" ("")
---
sig = 0x504b0102 (Central directory file header)
version = 0x003f (63)
version_needed = 0x000a (10)
flags = 0x0000 (0)
compression = 0x0000 (0)
mod_time = 0x48b3 ((9, 5, 38))
mod_date = 0x5119 ((2020, 8, 25))
crc32 = 0xd87f7e0c (3632233996)
compressed_size = 0x00000004 (4)
uncompressed_size = 0x00000004 (4)
file_name_length = 0x000a (10)
extra_field_length = 0x0024 (36)
file_comment_length = 0x0000 (0)
disk_number_start = 0x0000 (0)
internal_file_attributes = 0x0000 (0)
external_file_attributes = 0x00000020 (32)
lfh_offset = 0x00000772 (1906)
file_name = "7465737430322e747874" ("test02.txt")
extra_field = "0a002000000000000100180061a801cfe87ad601276b8a3c9\
86bd801034d8930986bd801"
file_comment = "" ("")
---
sig = 0x504b0506 (End of central directory record)
disk_number = 0x0000 (0)
disk_number_w_cd = 0x0000 (0)
disk_entries = 0x000d (13)
total_entries = 0x000d (13)
cd_size = 0x00000587 (1415)
cd_offset = 0x0000079e (1950)
comment_length = 0x0000 (0)
zip_file_comment = "" ("")
---
EOF
---

```

