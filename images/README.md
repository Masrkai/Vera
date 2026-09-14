# DISCLAMER: I do not own these images Nor Claim to be

the images were gracefully provided by

- the creators in this repo:
  - github.com/ianare/exif-samples
- from the exiv2 project samples webpage (was intended but not added yet)
  - exiv2.org/sample.html

And I would like to thank them for all the knowledge they provided me while learning the topic that made me interested in making this project

## the text files ?

they look a little uneasy for the eye but fair explination exists to why they exist
the project is built on top of the exiv2 utility and as such I wanted to test the flags properly
since I avoided doing so in the first versions of Vera so my experiment was:

```bash
exiv2 -pe 1.jpg > raw.txt
exiv2 -pt 1.jpg > translated.txt
exiv2 -pa 1.jpg > all.txt
```

and the attached files named exactly that are present for anyone to see.

## Patern ?

`-pe` and `-pt` are not equivalent outputs.
They contain mostly the same metadata keys, but they represent the values differently.

### `-pe`: raw/technical values

Example:

```
Exif.Photo.ExposureTime  Rational  1  4/300
Exif.Photo.FNumber       Rational  1  59/10
Exif.GPSInfo.GPSLatitude Rational  3  43/1 28/1 281400000/100000000
```

This output preserves the underlying encoded values:

- Fractions such as `4/300`
- Numeric codes such as `1`, `2`, `5`, and `16`
- GPS coordinates as degrees/minutes/seconds rationals
- Raw byte values such as `48 50 50 48`

### `-pt`: translated/human-readable values

Example:

```
Exif.Photo.ExposureTime  Rational  1  1/75 s
Exif.Photo.FNumber       Rational  1  F5.9
Exif.GPSInfo.GPSLatitude Rational  3  43 deg 28' 2.81"
```

This converts values into user-friendly representations:

- `4/300` → `1/75 s`
- `59/10` → `F5.9`
- `1` → `Auto`
- `16` → `No, compulsory`
- GPS rationals → formatted coordinates
- `2` → `inch`


For this image and these metadata fields,
`-pa` produces the same human-readable values as -pt, while also including an additional XMP field.


## annoyances

while the flags have diffrences they apparently have similarity of being equally annoying as hell
in some examples like:

```txt
Exif.Nikon3.DataDump Undefined 2542 <ALOT OF BINARY BLOB>
Exif.Photo.MakerNote Undefined 3298  <ALOT OF BINARY BLOB>
```

if parsed it will be completely unnecessary, while also being large and unusable


## conclusion

so the wisest option is to use `-pa`  

also mitigations should be placed like:

- checking if the label has `datadump` or `makernote` in it's name 
- checking if the type is Undefined and exceed a certain threshold a safe bet would be:

```rust
const MAX_DISPLAY_VALUES: usize = 256;
```
