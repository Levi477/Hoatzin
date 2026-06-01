def main(input):
    node1 = input.get("node1", {})
    node3 = input.get("node3", {})
    print(input)
    return {"result": node1.get("a", 0) + node1.get("b", 0) + node3.get("c", 0)}
