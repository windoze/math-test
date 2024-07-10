# Math practice for the junior school students

## Build from source
```bash
$ cd frontend
$ npm install # Needed only once unless you change the dependencies
$ cd ..
$ cargo build --release
```

## Run
Run the `server`, then open the `http://localhost:3001` in the browser.

Run `server --help` to see the available options.

## Build Docker image
```bash
$ docker build -t math-practice .
```

## Run Docker container
```bash
$ docker run -d -v /path/to/data:/data -p 3001:3001 math-practice
```
Where `/path/to/data` is the path to the directory with the `questions.db` file. Omit this option if you don't need to persist the data.
