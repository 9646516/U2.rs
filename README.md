## About

U2.rs is a cross platform solution for u2.dmhy with terminal UI and dashboard

## Usage

- install `Transmission`,make sure that transmission RPC can be accessed to

- prepare a `args.toml` in the same folder of the binaries

  it should be in the following format

  ```
  key1 = value1
  key2 = value2
  ...
  ```

  | Keys        | Value type | Optional | Description                                                  |
  | ----------- | ---------- | -------- | ------------------------------------------------------------ |
  | cookie      | String     | No       | cookie of the u2.dmhy,to be exact,the value of `nexusphp_u2`,which is a must for accessing to u2 |
  | proxy       | String     | Yes      | your proxy address,eg`"http://127.0.0.1:2333"`               |
  | workRoot    | String     | No       | absolute path of working directory                           |
  | RpcURL      | String     | No       | transmission RPC url,eg`"http://127.0.0.1:2333/transmission/rpc"` |
  | RpcUsername | String     | No       | transmission RPC username                                    |
  | RpcPassword | String     | No       | transmission RPC password                                    |
  | logRoot     | String     | No       | absolute path of logging directory                           |
  | maxSize     | Float      | No       | size limit of total size of downloaded files in GiB          |
  | downloadFx  | Float      | Yes      | downloadFx Filter                                            |

- Run the binaries

## Attention

the web crawling is based on the regex of `text`, which indicate you should use `Chinese` as UI language

## License 

[MIT](LICENSE)