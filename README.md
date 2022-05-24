# payment_engine
A simple toy transaction engine that can process transactions, deposits, withdrawals, deposits, resolves, and chargebacks. The engine updates client's accounts and outputs the state of the clients accounts as a CSV file.

## Usage

To run the payments engine enter the input file and output file like the example below:

`$ cargo run -- csv_files/tx.csv > accounts.csv`

if no output file is given the output will be printed to **stdout**

## Assumptions

* The input is a CSV file, it has max of four columns: `type`,`client`, `tx`, and `amount`.
* `type` is a string, 
* ` client` is a valid u16 client ID
* `tx` is a valid u32 transaction ID,
* `amount` is a decimal value with a precision of up to **four places past the decimal**.
* **Disputes will only work for deposits**.
* A transaction can be disputed/resolved many times, but **charged back only once**.
* If account is frozen all operations are blocked.

**Input example:**

| type       | client | tx   | amount      |
| ---------- | ------ | ---- | ----------- |
| deposit    | 35978  | 1    | 875.2435    |
| deposit    | 58598  | 2    | 891.4498    |
| withdrawal | 23187  | 3    | 3, 137.4155 |
| deposit    | 17526  | 4    | 546.5741    |
| deposit    | 17438  | 5    | 321.6327    |
| withdrawal | 39469  | 6    | 53.5526     |
| deposit    | 62104  | 7    | 102.2511    |
| dispute    | 62104  | 7    |             |
| chargeback | 62104  | 7    |             |

**Output example:**

| client | available | held   | total    | locked |
| ------ | --------- | ------ | -------- | ------ |
| 35978  | 875.2435  | 0.0000 | 875.2435 | false  |
| 23187  | 0.0000    | 0.0000 | 0.0000   | false  |
| 17526  | 546.5741  | 0.0000 | 546.5741 | false  |
| 17438  | 321.6327  | 0.0000 | 321.6327 | false  |
| 58598  | 891.4498  | 0.0000 | 891.4498 | false  |
| 39469  | 0.0000    | 0.0000 | 0.0000   | false  |
| 62104  | 0.0000    | 0.0000 | 0.0000   | true   |

## Design

* Used `Tokio` to achieve concurrency when processing transactions and displaying output
  * Used actor based model which uses two channels one as receiver for transactions and other to send the output 
  * In the current implementation the numbers of actors being spawn equal to number of files being read 
  * Also it uses a hashmap which holds all client ids and client struct instances which simulates accounts based on transaction from file, in this way is possible to have an internal state and mutate the account balance when a given transaction occurs multiple times on the same account

* The `Client` struct holds all the data important for an account like available balance, held balance, log of past transactions, and if the account is frozen.

