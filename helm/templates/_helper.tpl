{{/*
Sets tolerations for daemonsets either from the global var or from individual values
*/}}
{{- define "common.tolerations" }}
{{- $ctx := .context }}
{{- $component := .component }}
{{- $local := index $ctx.Values $component "tolerations" }}
{{- $global := $ctx.Values.global.tolerations }}
{{- if and (not (empty $local)) }}
tolerations:
{{ toYaml $local | indent 2 }}
{{- else if and (not (empty $global)) }}
tolerations:
{{ toYaml $global | indent 2 }}
{{- end }}
{{- end }}

{{/*
Sets priorityClassName for daemonsets either from the global var or from individual values
*/}}
{{- define "common.priorityClassName" }}
{{- $ctx := .context }}
{{- $component := .component }}
{{- $local := index $ctx.Values $component "priorityClassName" }}
{{- $global := $ctx.Values.global.priorityClassName }}
{{- if and (not (empty $local)) }}
priorityClassName: {{ toYaml $local }}
{{- else if and (not (empty $global)) }}
priorityClassName: {{ toYaml $global }}
{{- end }}
{{- end }}
