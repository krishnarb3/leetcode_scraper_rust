# Leetcode scraper

Fetches a random question from a set of specific companies and difficulties
> Requires Leetcode premium session token 

URL  of the problem is  output to console and/or posted as message to a discord webhook

### Usage (CLI)
```bash
 cargo run --package leetcode_scraper --bin bootstrap -- --companies company1 company2 --difficulties difficulty1 difficulty2
```

#### Arguments
Companies:
```bash
--companies 
```
As per the request made in the graphql request for fetching problems for a company (Eg: [https://leetcode.com/company/google/](https://leetcode.com/company/google/))

Difficulties:
```bash
--difficulties
```
One of <b>Easy</b>, <b>Medium</b> or <b>Hard</b>

### Example invocation: 
```bash
 cargo run --package leetcode_scraper --bin bootstrap -- --companies google facebook --difficulties Easy Medium
```

# Environment variables
### Leetcode session:
The value for leetcode session can be found in the cookie stored (The last key: LEETCODE_SESSION)
```bash
export LEETCODE_SESSION=value
```
### Discord (Optional):
Create a discord webhook for the specific channel and set the following variable
```bash
export DISCORD_WEBHOOK_URL_KEY=value
```

### Modes
#### CLI

#### AWS Lambda
Set the following environment variable:
```bash
export RUN_MODE="AWS_LAMBDA"
```

Building the zip for lambda:
```bash
docker run --rm -v $PWD:/code -v $HOME/.cargo/registry:/root/.cargo/registry -v $HOME/.cargo.git:/root/.cargo/git rustserverless/lambda-rust
```

Deploy lambda on AWS and set <b>Rule</b> using schedule entries (cron)