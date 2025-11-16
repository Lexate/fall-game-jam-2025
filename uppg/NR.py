def f(int):
    return int**2 - 1


x = [3.0]
while len(x) < 25:
    fk = f(x[-1])
    if abs(fk) < 1e-5:
        break
    x.append(x[-1] - fk**2 / (f(x[-1] + fk) - fk))
else:
    # Prints if the break was never hit
    print("Did not converge :/")
print(f"Converged to {round(x[-1], 4)} in {len(x)} interations")
