import React from "react";
import { Field, useField } from "formik";
import "./FormStyles.css";

interface FormInputProps {
  name: string;
  label: string;
  placeholder?: string;
  inputType?: "text" | "password";
  error?: React.ReactNode;
  marginBottom?: boolean;
}

const FormInput: React.FC<FormInputProps> = ({
  name,
  label,
  placeholder,
  inputType = "text",
  marginBottom,
}) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [field, meta, helpers] = useField(name);

  return (
    <div
      className="inputContainer"
      style={marginBottom ? { marginBottom: "1em" } : {}}
    >
      <label htmlFor={name}>{label}</label>
      <Field
        className="formField"
        name={name}
        type={inputType}
        placeholder={placeholder}
      />

      {meta.touched && meta.error ? (
        <div className="error">{meta.error}</div>
      ) : null}
    </div>
  );
};

export default FormInput;
