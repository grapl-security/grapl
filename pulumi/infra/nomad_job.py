import json
from pathlib import Path
from typing import Any, Mapping, Optional, Union, cast, get_args

import pulumi_nomad as nomad
from infra.config import STACK_NAME
from infra.kafka import NomadServiceKafkaCredentials
from infra.nomad_service_postgres import NomadServicePostgresDbArgs

import pulumi

_ValidNomadVarTypePrimitives = Union[str, bool, int]
_ValidNomadVarTypes = pulumi.Input[
    Union[
        _ValidNomadVarTypePrimitives,
        Mapping[str, pulumi.Input[_ValidNomadVarTypePrimitives]],
        Mapping[str, Any],
        # Mapping[str, DockerImageId]
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
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        vars = self._fix_pulumi_preview(vars)
        vars = self._json_dump_complex_types(vars)
        pulumi.info(f"{name} vars: {vars}")

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
        with open(nomad_file, "r") as f:
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

    def _fix_pulumi_preview(self, vars: NomadVars) -> NomadVars:
        """
        This is an ugly hack to deal with pulumi preview never resolving Outputs into a real string.
        Without this, the vars gets unset if there's a single key with an unresolved output
        """
        if pulumi.runtime.is_dry_run():
            pulumi_preview_replacement_string = "PULUMI_PREVIEW_STRING"
            # special rule since we string-split the redis endpoint
            redis_endpoint = "redis://some-fake-host-for-preview-only:1111"

            nomad_vars = {}
            for key, value in vars.items():

                if isinstance(value, pulumi.Output):
                    inner_type = value.__orig_class__.__args__[0].__name__
                    pulumi.info(
                        f"{key} is a pulumi output with inner type {inner_type}"
                    )
                    # This checks to see if this is a pulumi.Output[str] using _undocumented python implementation details_
                    # https://twitter.com/chompie1337/status/1435775022694555652?cxt=HHwWiICz0dXx8uwnAAAA
                    # SO thread on the python undocumented implementation details in question:
                    # https://stackoverflow.com/questions/57706180/generict-base-class-how-to-get-type-of-t-from-within-instance/60984681#60984681
                    #     if get_args(
                    #     value.__orig_class__
                    # ) == (str,):

                    # we're going to need to parse the hcl file to get the type for the variable
                    """
                    import hcl2
                    with open('foo.nomad', 'r') as file:
                        hcl2_dict = hcl2.load(file)
                        # flatten the list of dicts into a dict
                        variable_dict = {k:v for variable in hcl_dict['variable'] for k, v in variable.items()}
                        nomad_variable = variable_dict[key]:
                            if 'type' not in nomad_variable:
                                throw exception(f"{key} doesn't have a type defined in Nomad. "
                            else:
                                mock_nomad_variable # recursive function that walks the type definition and replaces strings, numbers etc
                                
                                
                                
                                def mock_nomad_variables(
                                    nomad_type = regex \$\{a+[\}\(]
                                    if nomad_type== "string"
                                        return MOCK_STRING
                                    if nomad_type == "number"
                                        return 7
                                    if nomad_type == object
                                        split
                                        for k,v in object:
                                            mock_nomad_variables
                            
                    """

                    value = pulumi_preview_replacement_string
                    if key == "redis_endpoint":
                        value = redis_endpoint

                nomad_vars[key] = value
            return nomad_vars
        else:
            return vars
