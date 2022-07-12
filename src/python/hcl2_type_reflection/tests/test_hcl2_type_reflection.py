import os
import unittest
from typing import Any

import hcl2
from hcl2_type_reflection.hcl2_type_reflection import HCL2TypeParser
from lark import Lark


class TestHCL2TypeReflection(unittest.TestCase):
    hcl2_type_dict: dict[str, Any]
    parser: Lark

    @classmethod
    def setUpClass(cls) -> None:
        nomad_path = os.path.join(os.path.dirname(__file__), "hcl2_type_test.nomad")
        cls.parser = HCL2TypeParser().parser
        with open(nomad_path) as file:
            hcl2_dict = hcl2.load(file)
            # flatten the list of dicts into a dict
            cls.hcl2_type_dict = {
                k: v for variable in hcl2_dict["variable"] for k, v in variable.items()
            }

    def test__reflect_string(self) -> None:
        type_variable = self.hcl2_type_dict["string_var"]["type"]
        parsed_type = self.parser.parse(type_variable)
        assert parsed_type == "string"

    def test__reflect_number(self) -> None:
        type_variable = self.hcl2_type_dict["number_var"]["type"]
        parsed_type = self.parser.parse(type_variable)
        assert parsed_type == "number"

    def test__reflect_map_string(self) -> None:
        type_variable = self.hcl2_type_dict["map_string_var"]["type"]
        parsed_type = self.parser.parse(type_variable)
        assert parsed_type == {"string": "string"}

    def test__reflect_object(self) -> None:
        type_variable = self.hcl2_type_dict["object_var"]["type"]
        parsed_type = self.parser.parse(type_variable)
        assert parsed_type == {
            "hostname": "string",
            "port": "number",
            "username": "string",
            "password": "string",
        }

    def test__reflect_map_object(self) -> None:
        type_variable = self.hcl2_type_dict["map_object_var"]["type"]
        parsed_type = self.parser.parse(type_variable)
        assert parsed_type == {
            "string": {"sasl_username": "string", "sasl_password": "string"}
        }
