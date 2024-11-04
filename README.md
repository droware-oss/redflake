# Redflake

Redflake is a distributed unique ID generator inspired by [Twitter's Snowflake](https://blog.twitter.com/2010/announcing-snowflake).
It serves IDs over the Redis Serialization Protocol (RESP), making it compatible with standard Redis clients.

| Component  | Bits    | Description                                        |
|------------|---------|----------------------------------------------------|
| Sign Bit   | 1 bit   | Always 0 to ensure the ID is a positive integer.   |
| Timestamp  | 43 bits | Milliseconds precision time since epoch.           |
| Machine ID | 8 bits  | Supports up to 256 independent nodes (0-255).      |
| Sequence   | 12 bits | Supports up to 4,096 IDs per millisecond per node. |

## Usage

### Terminal Example

```bash
$ redis-cli -h 127.0.0.1 -p 6380
127.0.0.1:6380> next
(integer) 863786482310053888
```

### Python Example

```python
from redis import Redis

r = redis.Redis(host='127.0.0.1', port=6380)
unique_id = r.execute_command('next')
print(f"Generated ID: {unique_id}")
```

## License

Mozilla Public License Version 2.0 (MPL-2.0 license)

See [LICENSE](https://github.com/droware-oss/redflake/blob/main/LICENSE) for details.
