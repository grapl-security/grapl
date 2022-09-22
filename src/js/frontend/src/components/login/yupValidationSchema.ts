import * as Yup from "yup";

export const yupValidationSchema = Yup.object().shape({
  username: Yup.string().required("Username Required"),
  password: Yup.string().required("Password Required"),
});
