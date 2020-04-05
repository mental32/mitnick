FROM rustlang/rust:nightly-slim

COPY . .

CMD [ "cargo", "run", "--", "run" ]
