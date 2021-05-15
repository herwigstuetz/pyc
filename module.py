
class Abc:
    def __init__(self):
        pass

    def configure(self):
        self.outports = {
            "out1": {
                "value": 1.0,
                "type": "float"
            }
        }

    def run(self):
        self.outports["out1"]["value"] += 1
