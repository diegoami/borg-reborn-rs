"""Design arithmetic only: exact boundary-model power, no game or seed access."""
import math

N = 900
ALPHA = 0.05
NET_GAIN = 0.10
for critical in range(N // 2 + 1, N + 1):
    null_tail = sum(math.comb(N, j) / 2**N for j in range(critical, N + 1))
    if 2 * null_tail <= ALPHA:
        break
win = (1 + NET_GAIN) / 2  # boundary model: every pair is discordant
power = sum(
    math.exp(math.lgamma(N + 1) - math.lgamma(j + 1) - math.lgamma(N - j + 1)
             + j * math.log(win) + (N - j) * math.log1p(-win))
    for j in range(N + 1) if j >= critical or j <= N - critical
)
print(f'{N} matched pairs; reject at wins >= {critical} or <= {N-critical}')
print(f'Boundary-model power at net gain {NET_GAIN:.2f}: {power:.4%}')
