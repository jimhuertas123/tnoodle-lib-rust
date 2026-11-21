package org.timepedia.exporter.client;

// Stand-in for the GWT exporter annotation; it has no effect on scramble logic.
public @interface Export {
    String value() default "";
}
