/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

package software.amazon.smithy.rust.codegen.smithy.protocols

import software.amazon.smithy.model.Model
import software.amazon.smithy.model.shapes.OperationShape
import software.amazon.smithy.model.traits.TimestampFormatTrait
import software.amazon.smithy.rust.codegen.rustlang.CargoDependency
import software.amazon.smithy.rust.codegen.rustlang.asType
import software.amazon.smithy.rust.codegen.rustlang.rust
import software.amazon.smithy.rust.codegen.rustlang.rustBlockTemplate
import software.amazon.smithy.rust.codegen.smithy.RuntimeType
import software.amazon.smithy.rust.codegen.smithy.generators.ProtocolConfig
import software.amazon.smithy.rust.codegen.smithy.generators.ProtocolGeneratorFactory
import software.amazon.smithy.rust.codegen.smithy.generators.ProtocolSupport
import software.amazon.smithy.rust.codegen.smithy.protocols.parse.JsonParserGenerator
import software.amazon.smithy.rust.codegen.smithy.protocols.parse.StructuredDataParserGenerator
import software.amazon.smithy.rust.codegen.smithy.protocols.serialize.JsonSerializerGenerator
import software.amazon.smithy.rust.codegen.smithy.protocols.serialize.StructuredDataSerializerGenerator
import software.amazon.smithy.rust.codegen.smithy.transformers.OperationNormalizer
import software.amazon.smithy.rust.codegen.smithy.transformers.RemoveEventStreamOperations

class RestJsonFactory : ProtocolGeneratorFactory<HttpBoundProtocolGenerator> {
    override fun buildProtocolGenerator(
        protocolConfig: ProtocolConfig
    ): HttpBoundProtocolGenerator = HttpBoundProtocolGenerator(protocolConfig, RestJson(protocolConfig))

    override fun transformModel(model: Model): Model {
        return OperationNormalizer(model).transformModel().let(RemoveEventStreamOperations::transform)
    }

    override fun support(): ProtocolSupport {
        return ProtocolSupport(
            requestSerialization = true,
            requestBodySerialization = true,
            responseDeserialization = true,
            errorDeserialization = true
        )
    }
}

class RestJson(private val protocolConfig: ProtocolConfig) : Protocol {
    private val runtimeConfig = protocolConfig.runtimeConfig

    override val httpBindingResolver: HttpBindingResolver =
        HttpTraitHttpBindingResolver(protocolConfig.model, "application/json", "application/json")

    override val defaultTimestampFormat: TimestampFormatTrait.Format = TimestampFormatTrait.Format.EPOCH_SECONDS

    override fun structuredDataParser(operationShape: OperationShape): StructuredDataParserGenerator {
        return JsonParserGenerator(protocolConfig, httpBindingResolver)
    }

    override fun structuredDataSerializer(operationShape: OperationShape): StructuredDataSerializerGenerator {
        return JsonSerializerGenerator(protocolConfig, httpBindingResolver)
    }

    override fun parseGenericError(operationShape: OperationShape): RuntimeType {
        return RuntimeType.forInlineFun("parse_generic_error", "json_deser") {
            it.rustBlockTemplate(
                "pub fn parse_generic_error(response: &#{Response}<#{Bytes}>) -> Result<#{Error}, #{JsonError}>",
                "Response" to RuntimeType.http.member("Response"),
                "Bytes" to RuntimeType.Bytes,
                "Error" to RuntimeType.GenericError(runtimeConfig),
                "JsonError" to CargoDependency.smithyJson(runtimeConfig).asType().member("deserialize::Error")
            ) {
                rust(
                    "#T::parse_generic_error(response)",
                    RuntimeType.jsonErrors(runtimeConfig)
                )
            }
        }
    }
}
