import json
from pathlib import Path
from typing import Any, Mapping, Optional, Union, cast

import hcl2
import pulumi_nomad as nomad
from hcl2_type_reflection.hcl2_type_reflection.hcl2_type_reflection import (
    HCL2TypeParser,
    mock_hcl2_type,
)
from infra.config import STACK_NAME
from infra.kafka import NomadServiceKafkaCredentials
from infra.nomad_service_postgres import NomadServicePostgresDbArgs

import pulumi

_ValidNomadVarTypePrimitives = Union[str, bool, int]
_ValidNomadVarTypes = pulumi.Input[
    Union[
        _ValidNomadVarTypePrimitives,
        Mapping[str, pulumi.Input[_ValidNomadVarTypePrimitives]],
        # Upsettingly, TypedDicts are a Mapping[str, object]
        NomadServicePostgresDbArgs,
        Union[
            pulumi.Input[NomadServiceKafkaCredentials],
            Mapping[str, pulumi.Input[NomadServiceKafkaCredentials]],
        ],
    ]
]
NomadVars = Mapping[str, Optional[_ValidNomadVarTypes]]


class NomadJob(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        jobspec: Path,
        vars: NomadVars,
        opts: pulumi.ResourceOptions | None = None,
    ) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        vars = self._fix_pulumi_preview(vars, jobspec)
        vars = self._json_dump_complex_types(vars)

        self.job = nomad.Job(
            resource_name=f"{STACK_NAME}-{name}-job",
            jobspec=self._file_contents(str(jobspec)),
            hcl2=nomad.JobHcl2Args(enabled=True, vars=vars),
            # Wait for all services to become healthy
            detach=False,
            # Purge job from Nomad servers after a `pulumi destroy`
            purge_on_destroy=True,
            opts=pulumi.ResourceOptions.merge(
                opts, pulumi.ResourceOptions(parent=self)
            ),
        )

        self.register_outputs({})

    def _file_contents(self, nomad_file: str) -> str:
        with open(nomad_file) as f:
            jobspec = f.read()
            f.close()
            return jobspec

    @staticmethod
    def _json_dump_complex_types(vars: NomadVars) -> NomadVars:
        """
        Nomad accepts input of lists and maps, but the Nomad/Pulumi plugin doesn't
        convert them correctly.
        """

        def escape_str_value(
            val: _ValidNomadVarTypePrimitives,
        ) -> _ValidNomadVarTypePrimitives:
            if isinstance(val, str):
                # Gotta do some annoying escaping when the object field contains "${}"
                return val.replace("${", "$${")
            return val

        def dump_value(val: Any) -> _ValidNomadVarTypePrimitives:
            if isinstance(val, list):
                return json.dumps(val)
            elif isinstance(val, dict):
                return json.dumps({k: escape_str_value(v) for (k, v) in val.items()})
            else:
                return cast(_ValidNomadVarTypePrimitives, val)

        return {
            k: pulumi.Output.from_input(v).apply(dump_value) for (k, v) in vars.items()
        }

    def _fix_pulumi_preview(
        self,
        vars: NomadVars,
        jobspec: Path,
    ) -> NomadVars:
        """
        This is a hack to deal with issues around pulumi preview.
        The Problem: Specifically, during pulumi preview, pulumi Output objects never resolve into strings or other types. This means that if a PR creates a new resource and then tries to use that resource's attributes in a Nomad variable, there's a type error because Pulumi object != string. Frustratingly, this manifests as a variable unset error for ALL Nomad variables in the file. This can also happen if a project is being pulumi'd up with no existing resources.

        The Solution:
        We're using reflection to parse the Nomad file's input variable types. We're then mocking the primitives types (str, bool, number) with fake value




        """
        if pulumi.runtime.is_dry_run():

            # special rule since we string-split the redis endpoint
            redis_endpoint = "redis://some-fake-host-for-preview-only:1111"

            hcl2_parser = HCL2TypeParser().parser
            with open(jobspec) as file:
                hcl2_dict = hcl2.load(file)
                # flatten the list of dicts into a dict
                hcl2_type_dict = {
                    k: v
                    for variable in hcl2_dict["variable"]
                    for k, v in variable.items()
                }

            nomad_vars = {}
            for key, value in vars.items():

                if isinstance(value, pulumi.Output):
                    # special cases
                    if key == "redis_endpoint":
                        value = redis_endpoint
                    else:
                        raw_type = hcl2_type_dict[key]["type"]
                        parsed_type = hcl2_parser.parse(raw_type)
                        # now we replace the strings
                        value = mock_hcl2_type(parsed_type)

                nomad_vars[key] = value
            return nomad_vars
        else:
            return vars
