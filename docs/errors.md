# Server Error Reference

This document is the canonical repair guide for coded `AppError` responses emitted by `server/`.

---

## `SYSTEM_TIME_ERROR`

- **HTTP Status**: `500 Internal Server Error`
- **Meaning**: `SystemTime::now().duration_since(UNIX_EPOCH)` returned a `SystemTimeError` because the host system clock is set earlier than `1970-01-01T00:00:00Z`.
- **How to Fix**:
  1. Check the host or container system clock (`date -u`).
  2. Ensure NTP / time synchronization is active on the host machine.
  3. In tests, verify no mocked or shifted `SystemTime` is earlier than `UNIX_EPOCH`.

---

## `TIMESTAMP_OVERFLOW`

- **HTTP Status**: `500 Internal Server Error`
- **Meaning**: The millisecond timestamp exceeded `i64::MAX`.
- **How to Fix**:
  1. Check the host or container system clock (`date -u`) for a far-future date.
