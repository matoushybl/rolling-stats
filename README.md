Rolling Stats
=============
> A crate implementing a simple circular buffer fed by raw data using the `std::io::Write` trait and outputing significant statistics about a window of the parsed data.

## Features
There is a feature, that allows enabling a bit slower solution to the problem of reconstruction partial data: `Reconstructor`. The feature is called `reconstructor`.

The `Reconstructor` is slower, as it involves one more copy of the remaining data buffer.

When this feature is not enabled, a better solution is utilized, as it works on the input data slice directly and takes care only of the incomplete data and preprocessing the input slice. This solution is developed as part of the `PartialDataBuffer` struct.

## Pain points, areas of improvements

* Rolling stats uses the VecDequeue as data storage, whereas a fixed size circular buffer might have been more performant.
* More testing.
* Better CI (cargo clippy, etc.)
* Use correct documentation notation.

## License
```
Copyright 2021 Datamole s.r.o, Matous Hybl

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
