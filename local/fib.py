def fib(n):
    if n == 0:
        return 0
    if n == 1:
        return 1
    return fib(n - 1) + fib(n - 2)

n = 28
result = fib(n)

print("=== Axiom Fibonacci Demo ===")
print(f"Calculating fib({n})...")
print(f"Result: {result}")

print()
print("Sequence:")

def print_seq(i, limit):
    if i == limit:
        return
    print(f"fib({i}) = {fib(i)}")
    print_seq(i + 1, limit)

print_seq(0, n + 1)

print()
print("Demo Complete.")
